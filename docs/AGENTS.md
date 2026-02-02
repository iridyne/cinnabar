# Cinnabar (朱砂) - Architecture & Design Documentation

## Project Overview

**Cinnabar** is a lightweight, offline-first, streaming speech-to-text input tool for Linux systems. The name "朱砂" (cinnabar) refers to the red mineral mercury sulfide, symbolizing the transformation of raw audio input into refined text output.

### Core Objectives

- **Offline-First**: All processing happens locally using ONNX models
- **Streaming**: Real-time partial results with automatic endpoint detection
- **Lightweight**: Minimal dependencies, optimized for edge devices
- **Linux-Native**: Built specifically for Linux audio stack (ALSA/PipeWire via cpal)

## Architecture

### Actor Model Design

Cinnabar implements a concurrent actor-like architecture with two primary threads communicating via bounded channels:

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

### Thread Responsibilities

#### Audio Capture Thread
- Captures raw audio from default microphone via `cpal`
- Converts multi-channel audio to mono by averaging channels
- Resamples from hardware sample rate (44100/48000Hz) to 16000Hz
- Sends resampled chunks to inference thread via bounded channel

#### Inference Thread
- Receives audio chunks from channel
- Feeds samples to Sherpa-ONNX OnlineRecognizer
- Displays partial results during speech
- Detects sentence endpoints and displays final results
- Resets recognizer state after each sentence

## Critical Technical Decisions

### 1. Audio Resampling Strategy

**Problem**: Paraformer model requires exactly 16000Hz sample rate, but most hardware defaults to 44100Hz or 48000Hz.

**Solution**: Implemented `LinearResampler` with the following characteristics:

- **Algorithm**: Linear interpolation between adjacent samples
- **Stateful**: Maintains buffer to handle fractional sample positions across chunks
- **Efficient**: Single-pass processing with minimal memory overhead

**Implementation Details**:
```
Input:  44100 Hz stream (or 48000 Hz)
Output: 16000 Hz stream
Ratio:  2.75625 (44100/16000) or 3.0 (48000/16000)

For each output sample at position i:
  src_idx = i * ratio
  idx0 = floor(src_idx)
  idx1 = idx0 + 1
  frac = src_idx - idx0
  output[i] = input[idx0] * (1 - frac) + input[idx1] * frac
```

**Why Not Hardware Resampling?**
- Not all audio devices support 16kHz natively
- Software resampling ensures consistent behavior across hardware
- Provides control over resampling quality

### 2. Channel Architecture

**Bounded Channel (Capacity: 100)**
- Prevents unbounded memory growth if inference lags
- Backpressure mechanism: audio thread blocks if channel full
- Trade-off: 100 chunks ≈ 6.25 seconds of audio buffer at 16kHz

### 3. Endpoint Detection

**Automatic Sentence Segmentation**:
- Enabled via `enable_endpoint: true` in recognizer config
- Model detects silence periods to determine sentence boundaries
- Triggers final result display and recognizer reset

**User Experience Flow**:
1. User speaks: `🤔 Thinking: 我觉得...` (partial, updates in-place)
2. User pauses: `✅ Final: 我觉得 Rust 很强。` (final, new line)
3. Recognizer resets, ready for next sentence

### 4. Model Selection

**Sherpa-ONNX Streaming Paraformer (Bilingual ZH-EN)**
- **Type**: Streaming ASR (not batch)
- **Languages**: Chinese + English
- **Quantization**: INT8 for reduced model size
- **Files Required**:
  - `encoder.int8.onnx`: Encoder network
  - `decoder.int8.onnx`: Decoder network
  - `tokens.txt`: Vocabulary mapping

**Why Paraformer?**
- Alibaba's production-grade model
- Excellent Chinese recognition accuracy
- Streaming-capable (incremental decoding)
- Lightweight INT8 quantization

## Component Breakdown

### Dependencies

| Crate | Purpose | Version |
|-------|---------|---------|
| `cpal` | Cross-platform audio I/O | 0.15 |
| `sherpa-onnx` | ONNX inference engine | 1.10 |
| `crossbeam-channel` | Lock-free MPSC channels | 0.5 |
| `anyhow` | Error handling | 1.0 |
| `clap` | CLI argument parsing | 4.5 |
| `ctrlc` | Signal handling | 3.4 |

### Key Structures

#### `LinearResampler`
```rust
struct LinearResampler {
    from_rate: f32,      // Source sample rate
    to_rate: f32,        // Target sample rate (16000)
    buffer: Vec<f32>,    // Stateful buffer for fractional samples
}
```

#### `Args`
```rust
struct Args {
    model_dir: PathBuf,  // Path to model files (default: ./models)
}
```

## Data Flow

### Audio Pipeline
```
Hardware Mic (44100Hz, Stereo)
    ↓
cpal InputCallback
    ↓
Mono Mixing (average channels)
    ↓
LinearResampler (44100Hz → 16000Hz)
    ↓
Bounded Channel (capacity: 100)
    ↓
OnlineRecognizer.accept_waveform()
    ↓
OnlineRecognizer.decode_stream()
    ↓
Display (partial/final results)
```

### State Machine
```
[IDLE] ──speech──> [DECODING] ──silence──> [ENDPOINT] ──reset──> [IDLE]
         ↓              ↓                       ↓
    (no output)   (partial text)         (final text)
```

## Performance Characteristics

### Latency
- **Audio Capture**: ~10ms (cpal buffer size)
- **Resampling**: <1ms per chunk
- **Inference**: ~50-100ms per decode (CPU-dependent)
- **Total**: ~60-110ms end-to-end latency

### Memory
- **Model Size**: ~40MB (INT8 quantized)
- **Runtime**: ~100MB (including audio buffers)
- **Channel Buffer**: ~6.25 seconds of audio

### CPU
- **Threads**: 2 (audio + inference)
- **ONNX Threads**: 4 (configurable via `num_threads`)
- **Target**: Single-core capable on modern CPUs

## Usage

### Setup
```bash
# Download model files
./setup_models.sh

# Build and run
cargo run --release

# Custom model directory
cargo run --release -- --model-dir /path/to/models
```

### Expected Output
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

## Future Enhancements

### Phase 2: Virtual Keyboard Integration
- Integrate with Linux input subsystem (`uinput`)
- Simulate keyboard events to type recognized text
- Hotkey activation (e.g., Ctrl+Space to start/stop)

### Phase 3: Advanced Features
- Voice activity detection (VAD) for power efficiency
- Multi-model support (Whisper, Conformer, etc.)
- Punctuation restoration
- Speaker diarization
- Custom vocabulary injection

### Phase 4: GUI
- System tray indicator
- Visual feedback for listening state
- Configuration panel
- Model management UI

## Design Philosophy

1. **Simplicity First**: CLI MVP before GUI complexity
2. **Offline Privacy**: No cloud dependencies, all local processing
3. **Rust Safety**: Memory safety without garbage collection overhead
4. **Linux Native**: Embrace Linux audio stack, don't abstract it away
5. **Production Ready**: Industry-grade code standards from day one

## Technical Constraints

- **Linux Only**: No Windows/macOS support (by design)
- **CPU Inference**: No GPU acceleration (yet)
- **Single Language Model**: One model loaded at a time
- **No VAD**: Continuous processing (power inefficient)

## References

- [Sherpa-ONNX Documentation](https://k2-fsa.github.io/sherpa/onnx/)
- [Paraformer Paper](https://arxiv.org/abs/2206.08317)
- [cpal Documentation](https://docs.rs/cpal/)
- [Linux Audio Stack](https://wiki.archlinux.org/title/Sound_system)

---

**Last Updated**: 2026-02-03  
**Version**: 0.1.0 (MVP)  
**Status**: Core Demo Complete