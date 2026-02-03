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
| `sherpa-rs` | ONNX inference engine (Rust bindings) | 0.6 |
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

### Phase 1.5: Audio Device Management

**Current Limitation**:
- Only uses system default input device (`default_input_device()`)
- No device enumeration or selection capability
- No support for multiple microphones
- No device hot-plug detection

**Planned Features**:
- [ ] List all available audio input devices
- [ ] CLI argument for device selection (`--device <index>` or `--device <name>`)
- [ ] Interactive device selection menu
- [ ] Display device capabilities (sample rates, channels)
- [ ] Device hot-plug monitoring and auto-reconnect
- [ ] Save preferred device in config file

**Implementation**:
```rust
// Enumerate devices
let devices: Vec<_> = host.input_devices()?.collect();
for (idx, device) in devices.iter().enumerate() {
    println!("设备 {}: {} ({:?})", idx, device.name()?, device.default_input_config()?);
}

// Select by index or name
let device = if let Some(idx) = args.device_index {
    host.input_devices()?.nth(idx).context("设备索引无效")?
} else if let Some(name) = args.device_name {
    host.input_devices()?.find(|d| d.name().ok() == Some(name)).context("设备名称无效")?
} else {
    host.default_input_device().context("未找到默认设备")?
};
```

**CLI Arguments**:
- `--list-devices`: List all available input devices and exit
- `--device <index>`: Use device by index (from `--list-devices`)
- `--device-name <name>`: Use device by name

**Priority**: Medium (improves usability for multi-microphone setups)

### Phase 2: Virtual Keyboard Integration

#### Input Method Implementation Analysis

**Option A: Wayland Input Method Protocol (input-method-v2)**

*Architecture*:
```
Cinnabar → zwp_input_method_v2 → Compositor → Target Application
```

*Requirements*:
- Implement `zwp_input_method_v2` protocol (not `text-input-v3`)
- Register as input method with compositor
- Handle input method lifecycle (activate/deactivate)
- Commit text via `commit_string` requests

*Pros*:
- No special permissions required (runs as user)
- Wayland-native, future-proof design
- Works across all Wayland compositors (GNOME, KDE, Sway)
- Proper integration with compositor input stack
- Supports pre-edit text (候选词显示)

*Cons*:
- **High complexity**: Protocol state machine management
- **Limited Rust ecosystem**: `wayland-protocols` doesn't include input-method-v2 by default
- **Compositor dependency**: Requires compositor support (not all support input-method-v2)
- **No X11 support**: Wayland-only solution
- **Debugging difficulty**: Protocol errors are hard to trace
- **Demo unfriendly**: Requires compositor configuration to register as input method

*Implementation Estimate*:
- Protocol bindings generation: Complex (manual `wayland-scanner` usage)
- State machine implementation: ~500-800 LOC
- Compositor registration: Requires system-level configuration
- **Verdict**: Too complex for MVP/demo phase

**Option B: uinput Virtual Keyboard (Recommended for MVP)**

*Architecture*:
```
Cinnabar → /dev/uinput → Kernel evdev → X11/Wayland → Target Application
```

*Requirements*:
- Access to `/dev/uinput` (input group membership or udev rule)
- Create virtual keyboard device via `evdev` crate
- Emit key events (press/release sequences)
- Handle Unicode input via compose sequences or XKB

*Pros*:
- **Simple implementation**: ~100-200 LOC
- **Mature Rust ecosystem**: `evdev` crate is well-maintained
- **Works everywhere**: X11, Wayland, even TTY
- **Easy debugging**: Can test with `evtest` tool
- **Demo-friendly**: Quick setup with `usermod -aG input $USER`
- **No compositor dependency**: Kernel-level solution

*Cons*:
- Requires group membership or udev rule (one-time setup)
- Not "Wayland-native" (but works perfectly on Wayland)
- Unicode input requires key sequence generation
- Potential security concern (can inject input to any application)

*Implementation Estimate*:
- Device creation: ~50 LOC
- Key event emission: ~100 LOC
- Unicode handling: ~50 LOC (via XKB compose)
- **Verdict**: Ideal for MVP, production-ready

#### Recommended Approach: uinput (Option B)

**Rationale**:
1. **MVP Priority**: Get working demo quickly
2. **Reliability**: Proven approach used by existing tools (ydotool, wtype)
3. **Compatibility**: Works on all Linux systems regardless of display server
4. **Maintainability**: Simple codebase, easy to debug

**Permission Setup**:
```bash
# One-time setup (requires logout/login)
sudo usermod -aG input $USER

# Or via udev rule (no logout required)
echo 'KERNEL=="uinput", GROUP="input", MODE="0660"' | sudo tee /etc/udev/rules.d/99-cinnabar.rules
sudo udevadm control --reload-rules && sudo udevadm trigger
```

**Tech Stack**:
- `evdev` (0.12) - Virtual keyboard device creation and event emission
- `x11rb` (0.13) or `global-hotkey` (0.6) - Hotkey listening (X11/Wayland agnostic)
- `tokio` (1.42) - Async runtime for non-blocking hotkey handling

**Future Migration Path**:
- Phase 2: Ship with uinput implementation
- Phase 3: Add optional Wayland input-method-v2 backend for users who prefer it
- Use runtime detection to choose backend automatically

### Phase 3: Advanced Features
- Voice activity detection (VAD) for power efficiency
- Multi-model support (Whisper, Conformer, etc.)
- Punctuation restoration
- Speaker diarization
- Custom vocabulary injection

**Potential Tech Stack**:
- `webrtc-vad` (0.4) - WebRTC VAD implementation (lightweight)
- `onnxruntime` (0.0.14) - Direct ONNX runtime for Silero VAD
- `whisper-rs` (0.12) - OpenAI Whisper Rust bindings
- `tokenizers` (0.20) - HuggingFace tokenizers for custom vocabulary
- `serde` + `serde_json` (1.0) - Configuration file serialization
- `notify` (7.0) - File system watching for model hot-reload

### Phase 4: GUI
- System tray indicator
- Visual feedback for listening state
- Configuration panel
- Model management UI

**Potential Tech Stack**:
- `egui` (0.30) - Immediate mode GUI (lightweight, ~2MB binary)
- `iced` (0.13) - Elm-inspired GUI framework (declarative)
- `gtk4-rs` (0.9) - GTK4 bindings (Linux-native look)
- `tauri` (2.1) - Web-based GUI with Rust backend
- `dioxus` (0.6) - React-like Rust GUI framework
- `tray-icon` (0.18) - System tray support
- `tokio` (1.42) - Async runtime for non-blocking UI

## Potential Challenges & Mitigation Strategies

### Audio Stack Compatibility

**Problem**: Linux audio landscape is fragmented (ALSA, PulseAudio, PipeWire, JACK)

**Risks**:
- `cpal` may not detect microphone on some systems
- Sample rate/format negotiation failures
- Permission issues accessing audio devices
- Latency variations across different audio servers

**Mitigation**:
- Test on multiple distributions (Arch, Ubuntu, Fedora)
- Provide fallback device selection via CLI argument
- Document audio group membership requirements (`usermod -aG audio $USER`)
- Add verbose audio debugging mode (`--audio-debug`)
- Consider direct ALSA/PipeWire bindings as fallback

### Model Management

**Problem**: 40MB+ model files not suitable for git, download reliability issues

**Risks**:
- Users may have slow/unreliable internet connections
- Model file corruption during download
- Version mismatches between code and models
- Storage constraints on embedded devices

**Mitigation**:
- Implement model checksum verification (SHA256)
- Support resumable downloads with progress bars
- Provide multiple mirror sources (GitHub Releases, Hugging Face, CDN)
- Add `--model-cache-dir` for custom storage locations
- Consider model quantization levels (INT8 vs FP16 vs FP32)

### Performance Bottlenecks

**Problem**: Real-time inference on low-end hardware

**Risks**:
- CPU can't keep up with 16kHz audio stream
- Memory pressure on systems with <2GB RAM
- Thermal throttling on fanless devices
- Battery drain on laptops

**Mitigation**:
- Implement adaptive quality modes (fast/balanced/accurate)
- Add CPU usage monitoring and warnings
- Support model hot-swapping to lighter variants
- Implement frame dropping with user notification
- Profile with `perf` and optimize hot paths

### Thread Synchronization Issues

**Problem**: Audio thread and inference thread coordination

**Risks**:
- Channel overflow causing audio drops
- Deadlocks during shutdown
- Race conditions in endpoint detection
- Memory leaks from unreleased streams

**Mitigation**:
- Add comprehensive unit tests for channel behavior
- Implement graceful shutdown with timeout
- Use `Arc<AtomicBool>` for cancellation (already done)
- Add memory leak detection in CI (`valgrind`, `heaptrack`)
- Stress test with long-running sessions (24h+)

### uinput Permission Requirements (Phase 2)

**Problem**: Virtual keyboard requires root or special permissions

**Risks**:
- Users reluctant to run as root
- `uinput` module not loaded by default
- SELinux/AppArmor blocking access
- Wayland security model restrictions

**Mitigation**:
- Create udev rules for non-root access: `/etc/udev/rules.d/99-cinnabar.rules`
- Provide systemd service template
- Document capability-based permissions (`CAP_SYS_ADMIN`)
- Consider D-Bus activation for privileged operations
- Implement Wayland input-method protocol as alternative

### Dependency Build Failures

**Problem**: `sherpa-rs` has complex native dependencies (ONNX Runtime)

**Risks**:
- CMake version mismatches
- Missing system libraries (protobuf, abseil)
- Cross-compilation difficulties
- Long build times (10+ minutes)

**Mitigation**:
- Provide pre-built binaries for common platforms
- Document exact dependency versions in `docs/BUILD.md`
- Use Docker for reproducible builds
- Consider static linking for distribution
- Add CI matrix for multiple distros

### Latency Accumulation

**Problem**: Multiple processing stages add latency

**Risks**:
- Audio callback → resampling → channel → inference → display
- User perceives lag between speech and text
- Endpoint detection delayed by buffering

**Mitigation**:
- Measure end-to-end latency with timestamps
- Optimize resampler buffer management
- Use smaller channel capacity (trade memory for latency)
- Profile with `tracing` spans
- Target <100ms P95 latency

### Model Accuracy Issues

**Problem**: Paraformer may struggle with accents, noise, domain-specific terms

**Risks**:
- Poor recognition for non-standard Mandarin
- Background noise causing false positives
- Technical jargon not in vocabulary
- Code-switching between Chinese and English

**Mitigation**:
- Implement custom vocabulary injection (Phase 3)
- Add noise suppression preprocessing
- Support multiple model backends (Whisper, Conformer)
- Provide confidence scores in output
- Allow user feedback loop for corrections

## Pre-Development Checklist

### Development Environment

- [ ] Rust 1.70+ with `rustfmt` and `clippy`
- [ ] CMake 3.20+ for sherpa-rs native build
- [ ] Audio testing setup (microphone, speakers)
- [ ] Multiple Linux VMs/containers (Arch, Ubuntu 22.04, Fedora 39)
- [ ] Performance profiling tools (`perf`, `flamegraph`, `heaptrack`)
- [ ] Audio analysis tools (`pavucontrol`, `qpwgraph`, `alsamixer`)

### Testing Infrastructure

- [ ] Unit test framework with `cargo test`
- [ ] Integration tests with recorded audio samples
- [ ] Benchmark suite with `criterion`
- [ ] CI/CD pipeline (GitHub Actions)
- [ ] Fuzzing setup for audio resampler (`cargo-fuzz`)
- [ ] Memory leak detection in CI

### Model Assets

- [ ] Model download automation (`setup_models.sh`)
- [ ] Checksum verification (SHA256)
- [ ] Multiple model variants (INT8, FP16, FP32)
- [ ] Model versioning strategy
- [ ] Fallback models for low-end hardware
- [ ] License compliance documentation

### Documentation

- [ ] Architecture documentation (this file)
- [ ] Build instructions for each distro
- [ ] Troubleshooting guide
- [ ] Performance tuning guide
- [ ] API documentation (if library mode)
- [ ] Contributing guidelines

### Security Considerations

- [ ] Audit dependencies with `cargo-audit`
- [ ] Review unsafe code blocks (resampler, FFI)
- [ ] Validate model file integrity
- [ ] Sandbox inference process (seccomp, landlock)
- [ ] Document privilege requirements
- [ ] Security policy for vulnerability reports

### Performance Baselines

- [ ] Establish latency targets (P50, P95, P99)
- [ ] Memory usage limits (<200MB steady state)
- [ ] CPU usage targets (<50% single core)
- [ ] Battery impact measurements
- [ ] Thermal characteristics on fanless devices

### Distribution Preparation

- [ ] Package for Arch AUR
- [ ] Debian/Ubuntu `.deb` package
- [ ] Fedora `.rpm` package
- [ ] Flatpak manifest
- [ ] AppImage build
- [ ] Static binary with musl

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

## Additional Tech Stack Considerations

### Testing & Quality Assurance
- `criterion` (0.5) - Benchmarking framework for performance regression testing
- `proptest` (1.5) - Property-based testing for audio resampling logic
- `mockall` (0.13) - Mock object generation for unit testing
- `rstest` (0.23) - Fixture-based testing framework
- `insta` (1.41) - Snapshot testing for recognition results

### Performance & Optimization
- `rayon` (1.10) - Data parallelism for batch processing
- `parking_lot` (0.12) - Faster synchronization primitives
- `mimalloc` (0.1) - Microsoft's high-performance allocator
- `flate2` (1.0) - Model compression/decompression
- `memmap2` (0.9) - Memory-mapped file I/O for large models

### Logging & Observability
- `tracing` (0.1) - Structured logging and diagnostics
- `tracing-subscriber` (0.3) - Log formatting and filtering
- `syslog` (7.0) - Linux syslog integration
- `metrics` (0.24) - Application metrics collection

### Configuration & Persistence
- `toml` (0.8) - Configuration file format
- `directories` (5.0) - XDG base directory specification
- `rusqlite` (0.32) - SQLite for recognition history/cache
- `config` (0.14) - Layered configuration management

### GPU Acceleration (Future)
- `ort` (2.0) - ONNX Runtime with CUDA/TensorRT support
- `cudarc` (0.12) - CUDA bindings for custom kernels
- `wgpu` (23.0) - WebGPU for cross-platform GPU compute

### Audio Processing Enhancements
- `rubato` (0.16) - High-quality audio resampling (alternative to LinearResampler)
- `dasp` (0.11) - Digital audio signal processing primitives
- `hound` (3.5) - WAV file I/O for debugging/testing
- `opus` (0.3) - Opus codec for audio compression

### IPC & Integration
- `dbus` (0.9) - D-Bus integration for desktop notifications
- `zbus` (5.1) - Modern async D-Bus library
- `libpulse-binding` (2.28) - Direct PulseAudio integration
- `pipewire` (0.8) - Native PipeWire API bindings

## References

- [Sherpa-ONNX Documentation](https://k2-fsa.github.io/sherpa/onnx/)
- [Paraformer Paper](https://arxiv.org/abs/2206.08317)
- [cpal Documentation](https://docs.rs/cpal/)
- [Linux Audio Stack](https://wiki.archlinux.org/title/Sound_system)

---

## Appendix A: FFI Migration & Debugging Lessons

### 问题背景

在将 Cinnabar 从 sherpa-rs 迁移到直接使用 sherpa-onnx C FFI 时，遇到了持续的段错误（SIGSEGV）问题。程序能够成功加载模型、创建流对象、检测麦克风，但在处理音频数据时立即崩溃。

### 根本原因

#### 1. FFI 生命周期管理错误（主要原因）

**问题代码**：
```rust
impl OnlineRecognizer {
    pub fn new(...) -> anyhow::Result<Self> {
        unsafe {
            let encoder_c = CString::new(encoder).unwrap();
            let decoder_c = CString::new(decoder).unwrap();
            let tokens_c = CString::new(tokens).unwrap();
            
            let config = SherpaOnnxOnlineRecognizerConfig {
                model_config: SherpaOnnxOnlineModelConfig {
                    paraformer: SherpaOnnxOnlineParaformerModelConfig {
                        encoder: encoder_c.as_ptr(),  // ❌ 悬空指针
                        decoder: decoder_c.as_ptr(),  // ❌ 悬空指针
                    },
                    tokens: tokens_c.as_ptr(),        // ❌ 悬空指针
                    ...
                },
                ...
            };
            
            let recognizer = SherpaOnnxCreateOnlineRecognizer(&config);
            Ok(Self { recognizer })  // ❌ CString 被销毁，指针失效
        }
    }
}
```

**问题分析**：
- `CString` 对象在函数结束时被销毁
- C API 持有的指针变成悬空指针
- 当 sherpa-onnx 尝试读取模型路径时触发段错误

**修复方案**：
```rust
pub struct OnlineRecognizer {
    recognizer: *mut SherpaOnnxOnlineRecognizer,
    _encoder: CString,    // ✅ 保持生命周期
    _decoder: CString,
    _tokens: CString,
    _provider: CString,
    _decoding: CString,
}
```

#### 2. Sherpa-onnx 内部重采样器崩溃（次要原因）

**问题现象**：
```
/home/runner/work/sherpa-onnx/sherpa-onnx/sherpa-onnx/csrc/features.cc:AcceptWaveformImpl:104 Creating a resampler:
   in_sample_rate: 44100
   output_sample_rate: 16000

fish: Job 1, 'cargo run --release' terminated by signal SIGSEGV
```

**问题分析**：
- Sherpa-onnx 的 C++ 重采样器在 Linux（特别是 Arch/CachyOS）上存在 ABI 兼容性问题
- 可能与 libc 版本差异或内存对齐问题相关

**修复方案**：
```rust
// 强制使用 16000Hz 单声道，让 PipeWire 处理重采样
let config = cpal::StreamConfig {
    channels: 1,
    sample_rate: cpal::SampleRate(16000),
    buffer_size: cpal::BufferSize::Default,
};
```

#### 3. FFI 结构体定义不完整

**问题代码**：
```rust
#[repr(C)]
pub struct SherpaOnnxOnlineRecognizerResult {
    pub text: *const c_char,  // ❌ 缺少其他字段
}
```

**正确定义**（来自 C API 头文件）：
```rust
#[repr(C)]
pub struct SherpaOnnxOnlineRecognizerResult {
    pub text: *const c_char,
    pub tokens: *const c_char,
    pub tokens_arr: *const *const c_char,
    pub timestamps: *const c_float,
    pub count: c_int,
    pub json: *const c_char,
}
```

### 关键经验教训

#### 1. FFI 生命周期管理
- **规则**：C API 持有的指针必须在整个使用期间保持有效
- **实践**：将 `CString` 存储在结构体中，而不是临时变量
- **检查**：使用 Valgrind 或 AddressSanitizer 检测悬空指针

#### 2. FFI 结构体定义
- **规则**：必须与 C API 头文件完全一致（字段数量、类型、顺序）
- **实践**：直接查看 `.h` 文件，不要依赖文档或猜测
- **工具**：考虑使用 `bindgen` 自动生成绑定

#### 3. 调试策略
```bash
# 1. 检查共享库符号
nm -D libsherpa-onnx-c-api.so | grep SherpaOnnx

# 2. 查看 C API 头文件
cat sherpa-onnx/c-api/c-api.h | grep -A 20 "typedef struct"

# 3. 使用 debug 模式获取更多信息
cargo build && ./target/debug/cinnabar

# 4. 检查库依赖
ldd target/release/cinnabar

# 5. 查看系统日志（需要 root）
sudo dmesg | tail -20 | grep segfault
```

#### 4. sherpa-rs 的局限性
- sherpa-rs 0.6.8 主要支持离线批处理 API（`OfflineStream`）
- 缺少真正的流式 API（`OnlineStream`）支持
- 需要直接使用 sherpa-onnx C FFI 才能实现真正的流式识别

#### 5. 平台特定问题
- Linux 音频栈（ALSA/PipeWire）与 C++ 库的 ABI 兼容性
- 优先让系统处理音频格式转换，避免使用第三方库的重采样器
- 在 Arch/CachyOS 等滚动发行版上特别注意 ABI 兼容性

### 未来改进建议

#### 短期
- [x] 添加音频配置回退机制（如果 16kHz 不支持，使用手动重采样）
- [x] 实现 rpath 自动配置，避免手动设置 `LD_LIBRARY_PATH`
- [x] 添加设备枚举和选择功能

#### 中期
- [ ] 使用 `bindgen` 自动生成 FFI 绑定
- [ ] 添加单元测试验证 FFI 调用
- [ ] 研究 sherpa-onnx 的线程安全性

#### 长期
- [ ] 考虑使用其他 ASR 引擎（Vosk、Whisper）作为备选
- [ ] 等待 sherpa-rs 官方添加流式 API 支持
- [ ] 贡献修复到上游 sherpa-onnx 项目

### 技术债务记录

| 问题 | 优先级 | 状态 | 位置 |
|------|--------|------|------|
| 手动设置 LD_LIBRARY_PATH | 高 | ✅ 已修复 | build.rs |
| 硬编码 16kHz 配置 | 中 | ✅ 已改进 | src/main.rs + src/resampler.rs |
| 缺少设备选择功能 | 中 | ✅ 已实现 | src/main.rs |
| FFI 绑定手动维护 | 低 | 待优化 | src/ffi/mod.rs |

---

**最后更新**: 2026-02-03  
**版本**: 0.1.1 (Phase 1.5 完成)  
**状态**: ✅ 核心功能可用，短期改进已完成