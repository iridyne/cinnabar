# Cinnabar (朱砂)

简体中文 | [English](README_EN.md)

> Lightweight, offline-first, streaming speech-to-text input tool for Linux

## Overview

**Cinnabar (朱砂)** is an offline speech input tool designed specifically for Linux systems, built with Rust, powered by Sherpa-ONNX inference engine and Alibaba's Paraformer model for real-time streaming speech recognition.

### Core Features

- 🔒 **Fully Offline** - All processing happens locally, no internet required, privacy-first
- ⚡ **Real-time Streaming** - Display results as you speak, with partial results and automatic sentence segmentation
- 🪶 **Lightweight & Efficient** - Minimal dependencies, optimized for edge devices
- 🐧 **Linux Native** - Deep integration with Linux audio stack (ALSA/PipeWire)
- 🇨🇳 **Bilingual Support** - Based on Paraformer bilingual model, supports Chinese-English mixed recognition

## Quick Start

### System Requirements

- **OS**: Linux (Arch/CachyOS recommended)
- **Rust**: 1.70+ (2021 edition)
- **Audio**: ALSA or PipeWire
- **CPU**: x86_64 architecture

### Installation

1. **Clone the repository**
```bash
git clone https://github.com/yourusername/cinnabar.git
cd cinnabar
```

2. **Download models**
```bash
./setup_models.sh
```

3. **Build and run**
```bash
cargo run --release
```

### Usage

#### Basic Usage

```bash
# Use default model directory (./models)
cargo run --release

# Specify custom model directory
cargo run --release -- --model-dir /path/to/models
```

#### Expected Output

```
🔥 Cinnabar (朱砂) - Streaming Speech-to-Text
Model: ./models
🎤 Microphone: 44100 Hz, 2 channels
✅ Model loaded. Listening...

Press Ctrl+C to stop...

🤔 Thinking: 我觉得
🤔 Thinking: 我觉得 Rust
🤔 Thinking: 我觉得 Rust 很强
✅ Final: 我觉得 Rust 很强。

👋 Goodbye!
```

## Technical Architecture

### Core Components

```
┌─────────────────┐         ┌──────────────────┐
│  Audio Thread   │         │ Inference Thread │
│                 │         │                  │
│  1. Capture     │         │  1. Receive      │
│  2. Mono Mix    │ ──────> │  2. Decode       │
│  3. Resample    │ Channel │  3. Display      │
│  4. Send        │         │  4. Endpoint     │
└─────────────────┘         └──────────────────┘
```

### Key Technologies

#### Audio Resampling

Hardware microphones typically output at 44100Hz or 48000Hz, but the Paraformer model strictly requires 16000Hz. Cinnabar implements `LinearResampler` for real-time linear interpolation resampling:

- **Algorithm**: Linear interpolation between adjacent samples
- **State Management**: Maintains buffer to handle fractional sample positions across chunks
- **Performance**: Single-pass processing with minimal memory overhead

#### Actor Concurrency Model

Uses `crossbeam-channel` to implement an actor-like architecture:
- **Audio Thread**: Capture → Resample → Send
- **Inference Thread**: Receive → Decode → Display
- **Bounded Channel**: Capacity 100, prevents unbounded memory growth

#### Automatic Sentence Segmentation

Enables built-in silence detection via `enable_endpoint: true`:
- Display partial results while speaking (`🤔 Thinking: ...`)
- Display final results when sentence ends (`✅ Final: ...`)
- Automatically reset recognizer state for next sentence

### Dependencies

| Dependency | Purpose | Version |
|------------|---------|---------|
| `cpal` | Cross-platform audio I/O | 0.15 |
| `sherpa-onnx` | ONNX inference engine | 1.10 |
| `crossbeam-channel` | Lock-free MPSC channels | 0.5 |
| `anyhow` | Error handling | 1.0 |
| `clap` | CLI argument parsing | 4.5 |
| `ctrlc` | Signal handling | 3.4 |

## Performance Metrics

- **Latency**: 60-110ms end-to-end (CPU-dependent)
- **Memory**: ~100MB runtime (including audio buffers)
- **Model Size**: ~40MB (INT8 quantized)
- **CPU**: Single-core capable (modern CPUs)

## Roadmap

### Phase 1: CLI Core Demo ✅
- [x] Audio capture and resampling
- [x] Streaming speech recognition
- [x] Automatic sentence segmentation
- [x] Terminal output

### Phase 2: Virtual Keyboard Integration
- [ ] Linux `uinput` integration
- [ ] Simulate keyboard events
- [ ] Hotkey activation (e.g., Ctrl+Space)

### Phase 3: Advanced Features
- [ ] Voice Activity Detection (VAD)
- [ ] Multi-model support (Whisper, Conformer)
- [ ] Punctuation restoration
- [ ] Custom vocabulary injection

### Phase 4: GUI
- [ ] System tray indicator
- [ ] Visual listening state feedback
- [ ] Configuration panel
- [ ] Model management UI

## Documentation

- [Architecture Design Document](docs/AGENTS.md) - Detailed technical architecture and design decisions

## Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details

## Acknowledgments

- [Sherpa-ONNX](https://k2-fsa.github.io/sherpa/onnx/) - High-performance ONNX inference engine
- [Alibaba Paraformer](https://arxiv.org/abs/2206.08317) - Excellent Chinese speech recognition model
- [cpal](https://github.com/RustAudio/cpal) - Rust cross-platform audio library

---

**Version**: 0.1.0 (MVP)  
**Status**: Core Demo Complete  
**Last Updated**: 2026-02-03