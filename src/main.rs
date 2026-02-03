mod config;
mod ffi;
mod gui;
mod injector;
mod recognizer;
mod resampler;
mod vad;
mod wayland;

use anyhow::{Context, Result};
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::bounded;
use ffi::OnlineRecognizer;
use resampler::LinearResampler;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(name = "cinnabar")]
#[command(about = "轻量级、离线优先的 Linux 流式语音转文字工具")]
pub struct Args {
    /// 运行模式：cli 或 gui
    #[arg(short, long, default_value = "cli")]
    mode: String,

    #[arg(short = 'M', long, default_value = "./models")]
    model_dir: PathBuf,

    #[arg(short, long)]
    pub config: Option<PathBuf>,

    #[arg(long)]
    list_devices: bool,

    #[arg(short, long)]
    device: Option<usize>,

    #[arg(long)]
    device_name: Option<String>,

    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 模式切换
    match args.mode.as_str() {
        "gui" => return gui::run_gui_mode(&args),
        "cli" => {} // 继续执行 CLI 模式
        _ => anyhow::bail!("无效的模式。使用 'cli' 或 'gui'"),
    }

    // CLI 模式
    let host = cpal::default_host();

    if args.list_devices {
        println!("可用的音频输入设备：\n");
        for (idx, device) in host.input_devices()?.enumerate() {
            let name = device.name().unwrap_or_else(|_| "未知设备".to_string());
            let config = device.default_input_config();
            match config {
                Ok(cfg) => println!(
                    "  [{}] {} - {} Hz, {} 声道",
                    idx,
                    name,
                    cfg.sample_rate().0,
                    cfg.channels()
                ),
                Err(_) => println!("  [{}] {} - 无法获取配置", idx, name),
            }
        }
        return Ok(());
    }

    if !args.model_dir.exists() {
        anyhow::bail!("未找到模型目录：{}", args.model_dir.display());
    }

    let recognizer = OnlineRecognizer::new(
        &args.model_dir.join("encoder.int8.onnx").to_string_lossy(),
        &args.model_dir.join("decoder.int8.onnx").to_string_lossy(),
        &args.model_dir.join("tokens.txt").to_string_lossy(),
        4,
    )?;

    let mut stream = recognizer.create_stream();

    let device = if let Some(idx) = args.device {
        host.input_devices()?
            .nth(idx)
            .context(format!("设备索引 {} 无效", idx))?
    } else if let Some(name) = &args.device_name {
        host.input_devices()?
            .find(|d| d.name().ok().as_ref() == Some(name))
            .context(format!("未找到设备名称: {}", name))?
    } else {
        host.default_input_device().context("未找到默认输入设备")?
    };

    println!(
        "🎤 使用设备: {}",
        device.name().unwrap_or_else(|_| "未知设备".to_string())
    );

    // 尝试配置 16000Hz 单声道，如果不支持则使用默认配置并启用重采样
    let target_sample_rate = 16000;

    // 检查设备是否支持 16kHz 单声道配置
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
        println!("🔧 使用配置: 16000 Hz, 1 声道");
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
        let sample_rate = default_config.sample_rate().0;
        println!(
            "⚠️  16kHz 不支持，使用默认配置: {} Hz, {} 声道（将启用重采样）",
            sample_rate,
            default_config.channels()
        );
        (
            cpal::StreamConfig {
                channels: default_config.channels(),
                sample_rate: default_config.sample_rate(),
                buffer_size: cpal::BufferSize::Default,
            },
            sample_rate != target_sample_rate,
        )
    };

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    ctrlc::set_handler(move || {
        running_clone.store(false, Ordering::Relaxed);
    })?;

    let (tx, rx) = bounded::<Vec<f32>>(100);
    let actual_sample_rate = config.sample_rate.0;
    let channels = config.channels;
    let verbose = args.verbose;

    let audio_stream = device.build_input_stream(
        &config,
        move |data: &[f32], _| {
            if verbose {
                eprintln!("[DEBUG] 音频回调: 接收到 {} 个样本", data.len());
            }
            let mono_data: Vec<f32> = if channels > 1 {
                data.chunks(channels as usize)
                    .map(|chunk| {
                        let sum: f32 = chunk.iter().sum();
                        // 使用 sqrt(channels) 作为除数，避免音量过小
                        sum / (channels as f32).sqrt()
                    })
                    .collect()
            } else {
                data.to_vec()
            };
            if verbose {
                eprintln!("[DEBUG] 音频回调: 混音后 {} 个样本", mono_data.len());
            }
            let _ = tx.try_send(mono_data);
        },
        |err| eprintln!("错误：{}", err),
        None,
    )?;

    audio_stream.play()?;

    println!("开始监听... 按 Ctrl+C 停止");

    let mut resampler = if use_resampler {
        Some(LinearResampler::new(actual_sample_rate, target_sample_rate))
    } else {
        None
    };

    let mut last_result = String::new();

    while running.load(Ordering::Relaxed) {
        if let Ok(samples) = rx.recv_timeout(std::time::Duration::from_millis(100)) {
            if args.verbose {
                eprintln!("[DEBUG] 主循环: 接收到 {} 个样本", samples.len());
            }
            if samples.is_empty() {
                continue;
            }

            if args.verbose {
                eprintln!("[DEBUG] 主循环: 开始重采样");
            }
            let resampled = if let Some(ref mut r) = resampler {
                r.resample(&samples)
            } else {
                samples
            };
            if args.verbose {
                eprintln!("[DEBUG] 主循环: 重采样后 {} 个样本", resampled.len());
            }

            // 检查重采样后的数据是否为空
            if resampled.is_empty() {
                continue;
            }

            if args.verbose {
                eprintln!("[DEBUG] 主循环: 调用 accept_waveform");
            }
            stream.accept_waveform(target_sample_rate as i32, &resampled);

            if args.verbose {
                eprintln!("[DEBUG] 主循环: 检查 is_ready");
            }
            while recognizer.is_ready(&stream) {
                if args.verbose {
                    eprintln!("[DEBUG] 主循环: 调用 decode");
                }
                recognizer.decode(&mut stream);
            }

            if args.verbose {
                eprintln!("[DEBUG] 主循环: 获取结果");
            }
            let result = recognizer.get_result(&stream);
            let trimmed = result.trim();

            if !trimmed.is_empty() && trimmed != last_result {
                // 检测句子结束标点
                let has_sentence_end = trimmed.ends_with('。')
                    || trimmed.ends_with('？')
                    || trimmed.ends_with('！')
                    || trimmed.ends_with('.')
                    || trimmed.ends_with('?')
                    || trimmed.ends_with('!');

                if has_sentence_end {
                    println!("{}", trimmed);
                    last_result.clear();
                } else {
                    last_result = trimmed.to_string();
                }
            }

            if args.verbose {
                eprintln!("[DEBUG] 主循环: 检查 endpoint");
                eprintln!("[DEBUG] 主循环: 准备调用 is_endpoint 函数");
            }
            let is_endpoint = recognizer.is_endpoint(&stream);
            if args.verbose {
                eprintln!("[DEBUG] 主循环: is_endpoint 函数调用完成");
                eprintln!("[DEBUG] 主循环: endpoint = {}", is_endpoint);
            }
            if is_endpoint {
                if args.verbose {
                    eprintln!("[DEBUG] 主循环: endpoint 为 true，获取最终结果");
                }
                let final_result = recognizer.get_result(&stream);
                if args.verbose {
                    eprintln!(
                        "[DEBUG] 主循环: 获取到最终结果，长度 = {}",
                        final_result.len()
                    );
                }
                if !final_result.trim().is_empty() {
                    println!("\n✅ {}", final_result.trim());
                }
                if args.verbose {
                    eprintln!("[DEBUG] 主循环: 准备重置流");
                }
                recognizer.reset(&mut stream);
                if args.verbose {
                    eprintln!("[DEBUG] 主循环: 流已重置");
                }
            }
            if args.verbose {
                eprintln!("[DEBUG] 主循环: 本次循环结束");
            }
        }
    }

    Ok(())
}
