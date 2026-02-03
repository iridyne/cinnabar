use crate::ffi::OnlineRecognizer;
use crate::resampler::LinearResampler;
use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait};
use crossbeam_channel::{bounded, Receiver};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct RecognizerEngine {
    recognizer: OnlineRecognizer,
    _stream: cpal::Stream,
    rx: Receiver<Vec<f32>>,
    running: Arc<AtomicBool>,
    resampler: Option<LinearResampler>,
    target_sample_rate: u32,
}

impl RecognizerEngine {
    pub fn new(
        model_dir: &std::path::Path,
        device_idx: Option<usize>,
        device_name: Option<String>,
    ) -> Result<Self> {
        let recognizer = OnlineRecognizer::new(
            &model_dir.join("encoder.int8.onnx").to_string_lossy(),
            &model_dir.join("decoder.int8.onnx").to_string_lossy(),
            &model_dir.join("tokens.txt").to_string_lossy(),
            4,
        )?;

        let host = cpal::default_host();
        let device = if let Some(idx) = device_idx {
            host.input_devices()?
                .nth(idx)
                .context(format!("设备索引 {} 无效", idx))?
        } else if let Some(name) = &device_name {
            host.input_devices()?
                .find(|d| d.name().ok().as_ref() == Some(name))
                .context(format!("未找到设备名称: {}", name))?
        } else {
            host.default_input_device().context("未找到默认输入设备")?
        };

        let target_sample_rate = 16000;
        let supports_16khz = device
            .supported_input_configs()
            .ok()
            .and_then(|configs| {
                configs.filter(|c| c.channels() == 1).find(|c| {
                    let min = c.min_sample_rate().0;
                    let max = c.max_sample_rate().0;
                    target_sample_rate >= min && target_sample_rate <= max
                })
            })
            .is_some();

        let (config, use_resampler) = if supports_16khz {
            (
                cpal::StreamConfig {
                    channels: 1,
                    sample_rate: cpal::SampleRate(target_sample_rate),
                    buffer_size: cpal::BufferSize::Default,
                },
                false,
            )
        } else {
            let default_config = device.default_input_config()?;
            (
                cpal::StreamConfig {
                    channels: default_config.channels(),
                    sample_rate: default_config.sample_rate(),
                    buffer_size: cpal::BufferSize::Default,
                },
                default_config.sample_rate().0 != target_sample_rate,
            )
        };

        let (tx, rx) = bounded::<Vec<f32>>(100);
        let channels = config.channels;

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _| {
                let mono_data: Vec<f32> = if channels > 1 {
                    data.chunks(channels as usize)
                        .map(|chunk| chunk.iter().sum::<f32>() / (channels as f32).sqrt())
                        .collect()
                } else {
                    data.to_vec()
                };
                let _ = tx.try_send(mono_data);
            },
            |err| eprintln!("错误：{}", err),
            None,
        )?;

        let resampler = if use_resampler {
            Some(LinearResampler::new(
                config.sample_rate.0,
                target_sample_rate,
            ))
        } else {
            None
        };

        Ok(Self {
            recognizer,
            _stream: stream,
            rx,
            running: Arc::new(AtomicBool::new(false)),
            resampler,
            target_sample_rate,
        })
    }

    pub fn process(&mut self, stream: &mut crate::ffi::OnlineStream) -> Option<String> {
        if !self.running.load(Ordering::Relaxed) {
            return None;
        }

        if let Ok(samples) = self.rx.try_recv() {
            if samples.is_empty() {
                return None;
            }

            let resampled = if let Some(ref mut r) = self.resampler {
                r.resample(&samples)
            } else {
                samples
            };

            if resampled.is_empty() {
                return None;
            }

            stream.accept_waveform(self.target_sample_rate as i32, &resampled);

            while self.recognizer.is_ready(stream) {
                self.recognizer.decode(stream);
            }

            let result = self.recognizer.get_result(stream);
            let trimmed = result.trim();

            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }

        None
    }

    pub fn create_stream(&self) -> crate::ffi::OnlineStream {
        self.recognizer.create_stream()
    }
}
