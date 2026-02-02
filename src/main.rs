use anyhow::{Context, Result};
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{bounded, Sender};
use sherpa_onnx::{OnlineRecognizer, OnlineRecognizerConfig};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(name = "cinnabar")]
#[command(about = "Lightweight, offline-first, streaming speech-to-text for Linux", long_about = None)]
struct Args {
    #[arg(short, long, default_value = "./models")]
    model_dir: PathBuf,
}

struct LinearResampler {
    from_rate: f32,
    to_rate: f32,
    buffer: Vec<f32>,
}

impl LinearResampler {
    fn new(from_rate: u32, to_rate: u32) -> Self {
        Self {
            from_rate: from_rate as f32,
            to_rate: to_rate as f32,
            buffer: Vec::new(),
        }
    }

    fn resample(&mut self, input: &[f32]) -> Vec<f32> {
        if (self.from_rate - self.to_rate).abs() < 1.0 {
            return input.to_vec();
        }

        self.buffer.extend_from_slice(input);
        let ratio = self.from_rate / self.to_rate;
        let output_len = (self.buffer.len() as f32 / ratio) as usize;
        let mut output = Vec::with_capacity(output_len);

        for i in 0..output_len {
            let src_idx = i as f32 * ratio;
            let idx0 = src_idx.floor() as usize;
            let idx1 = (idx0 + 1).min(self.buffer.len() - 1);
            let frac = src_idx - idx0 as f32;

            if idx0 < self.buffer.len() {
                let sample = self.buffer[idx0] * (1.0 - frac) + self.buffer[idx1] * frac;
                output.push(sample);
            }
        }

        let consumed = (output_len as f32 * ratio) as usize;
        self.buffer.drain(..consumed.min(self.buffer.len()));

        output
    }
}

fn audio_capture_thread(tx: Sender<Vec<f32>>, running: Arc<AtomicBool>) -> Result<()> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .context("No input device available")?;

    let config = device
        .default_input_config()
        .context("Failed to get default input config")?;

    let sample_rate = config.sample_rate().0;
    let channels = config.channels() as usize;

    println!("🎤 Microphone: {} Hz, {} channels", sample_rate, channels);

    let mut resampler = LinearResampler::new(sample_rate, 16000);

    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mono: Vec<f32> = data
                .chunks(channels)
                .map(|frame| frame.iter().sum::<f32>() / channels as f32)
                .collect();

            let resampled = resampler.resample(&mono);
            if !resampled.is_empty() {
                let _ = tx.send(resampled);
            }
        },
        |err| eprintln!("Audio stream error: {}", err),
        None,
    )?;

    stream.play()?;

    while running.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    Ok(())
}

fn inference_thread(
    rx: crossbeam_channel::Receiver<Vec<f32>>,
    model_dir: PathBuf,
    running: Arc<AtomicBool>,
) -> Result<()> {
    let config = OnlineRecognizerConfig {
        model_config: sherpa_onnx::OnlineModelConfig {
            paraformer: sherpa_onnx::OnlineParaformerModelConfig {
                encoder: model_dir
                    .join("encoder.int8.onnx")
                    .to_string_lossy()
                    .to_string(),
                decoder: model_dir
                    .join("decoder.int8.onnx")
                    .to_string_lossy()
                    .to_string(),
            },
            tokens: model_dir.join("tokens.txt").to_string_lossy().to_string(),
            num_threads: 4,
            provider: "cpu".to_string(),
            debug: false,
            ..Default::default()
        },
        enable_endpoint: true,
        max_active_paths: 4,
        ..Default::default()
    };

    let recognizer = OnlineRecognizer::new(config).context("Failed to create recognizer")?;
    let mut stream = recognizer.create_stream();

    println!("✅ Model loaded. Listening...\n");

    let mut last_text = String::new();

    while running.load(Ordering::Relaxed) {
        match rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(samples) => {
                stream.accept_waveform(16000, &samples);

                while recognizer.is_ready(&stream) {
                    recognizer.decode_stream(&mut stream);
                }

                let result = recognizer.get_result(&stream);
                let text = result.text.trim();

                if !text.is_empty() && text != last_text {
                    print!("\r🤔 Thinking: {}", text);
                    std::io::Write::flush(&mut std::io::stdout()).ok();
                    last_text = text.to_string();
                }

                if recognizer.is_endpoint(&stream) {
                    let final_result = recognizer.get_result(&stream);
                    let final_text = final_result.text.trim();

                    if !final_text.is_empty() {
                        println!("\r✅ Final: {}\n", final_text);
                        last_text.clear();
                    }

                    recognizer.reset(&mut stream);
                }
            }
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    if !args.model_dir.exists() {
        anyhow::bail!(
            "Model directory not found: {}\nRun ./setup_models.sh first",
            args.model_dir.display()
        );
    }

    println!("🔥 Cinnabar (朱砂) - Streaming Speech-to-Text");
    println!("Model: {}", args.model_dir.display());

    let (tx, rx) = bounded(100);
    let running = Arc::new(AtomicBool::new(true));

    let running_audio = running.clone();
    let audio_thread = std::thread::spawn(move || audio_capture_thread(tx, running_audio));

    let running_inference = running.clone();
    let model_dir = args.model_dir.clone();
    let inference_thread =
        std::thread::spawn(move || inference_thread(rx, model_dir, running_inference));

    println!("Press Ctrl+C to stop...\n");

    ctrlc::set_handler(move || {
        running.store(false, Ordering::Relaxed);
    })
    .context("Failed to set Ctrl+C handler")?;

    audio_thread.join().unwrap()?;
    inference_thread.join().unwrap()?;

    println!("\n👋 Goodbye!");

    Ok(())
}
