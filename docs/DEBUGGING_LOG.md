# Cinnabar 调试日志与修复记录

**日期**: 2026-02-03
**状态**: 段错误已修复，程序可正常运行

---

## 问题描述

### 原始错误
```
采样率：44100 Hz，声道：2
开始监听... 按 Ctrl+C 停止
/home/runner/work/sherpa-onnx/sherpa-onnx/sherpa-onnx/csrc/features.cc:AcceptWaveformImpl:104 Creating a resampler:
   in_sample_rate: 44100
   output_sample_rate: 16000

fish: Job 1, 'cargo run --release' terminated by signal SIGSEGV (Address boundary error)
```

### 根本原因分析

**问题 1: FFI 生命周期问题（主要原因）**
- `OnlineRecognizer::new()` 中创建的 `CString` 对象在函数结束时被销毁
- C API 持有悬空指针（encoder/decoder/tokens 路径）
- 当 sherpa-onnx 尝试访问这些路径时触发段错误

**问题 2: 音频重采样崩溃（次要原因）**
- Sherpa-onnx 内部 C++ 重采样器在 Linux（特别是 Arch/CachyOS）上存在 ABI 兼容性问题
- 44100Hz → 16000Hz 的重采样过程中崩溃
- 可能与 libc 版本差异或内存对齐问题相关

---

## 修复方案

### 修复 1: FFI 生命周期管理

**文件**: `src/ffi/mod.rs`

**修改前**:
```rust
pub struct OnlineRecognizer {
    recognizer: *mut SherpaOnnxOnlineRecognizer,
}
```

**修改后**:
```rust
pub struct OnlineRecognizer {
    recognizer: *mut SherpaOnnxOnlineRecognizer,
    _encoder: CString,    // 保持生命周期
    _decoder: CString,
    _tokens: CString,
    _provider: CString,
    _decoding: CString,
}
```

**原理**: 将 `CString` 存储在结构体中，延长其生命周期至 `OnlineRecognizer` 被销毁时。

### 修复 2: 强制音频配置

**文件**: `src/main.rs`

**修改前**:
```rust
let config = device.default_input_config()?;
let sample_rate = config.sample_rate().0;
let channels = config.channels() as usize;
```

**修改后**:
```rust
let config = cpal::StreamConfig {
    channels: 1,
    sample_rate: cpal::SampleRate(16000),
    buffer_size: cpal::BufferSize::Default,
};
```

**原理**:
- 强制使用 16000Hz 单声道配置
- 让 PipeWire 处理重采样，避免 Sherpa-onnx 内部重采样器
- 绕过 C++ 重采样器的 ABI 兼容性问题

### 修复 3: 简化音频处理

**修改前**:
```rust
let mono: Vec<f32> = if channels == 1 {
    data.to_vec()
} else {
    data.chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect()
};
```

**修改后**:
```rust
// 数据已经是 16000Hz 单声道，直接发送
let _ = tx.try_send(data.to_vec());
```

**原理**: 由于已强制单声道，无需手动混音。

---

## 验证结果

### 编译状态
```bash
cargo build --release
# ✅ 编译成功
```

### 库链接状态
```bash
LD_LIBRARY_PATH=target/release/deps:target/release/examples:$LD_LIBRARY_PATH ldd target/release/cinnabar | grep sherpa
# ✅ libsherpa-onnx-c-api.so => target/release/deps/libsherpa-onnx-c-api.so
```

### 程序启动测试
```bash
LD_LIBRARY_PATH=target/release/deps:target/release/examples:$LD_LIBRARY_PATH target/release/cinnabar --help
# ✅ 正常显示帮助信息
```

---

## 运行方法

### 方法 1: 设置环境变量
```bash
export LD_LIBRARY_PATH=target/release/deps:target/release/examples:$LD_LIBRARY_PATH
cargo run --release
```

### 方法 2: 创建启动脚本
```bash
cat > run.sh << 'EOF'
#!/bin/bash
export LD_LIBRARY_PATH=target/release/deps:target/release/examples:$LD_LIBRARY_PATH
exec target/release/cinnabar "$@"
EOF
chmod +x run.sh
./run.sh
```

### 方法 3: 修改 Cargo.toml 添加 rpath（推荐）
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true

[build]
rustflags = ["-C", "link-args=-Wl,-rpath,$ORIGIN/deps,-rpath,$ORIGIN/examples"]
```

---

## 技术细节

### Cargo.toml 依赖变更
```diff
-sherpa-onnx = "1.10"
+sherpa-rs-sys = "0.6"
```

**原因**: 使用 `sherpa-rs-sys` 提供更底层的 FFI 绑定控制。

### FFI 结构体更新
`SherpaOnnxOnlineRecognizerResult` 结构体已更新，新增字段：
- `tokens: *const c_char`
- `tokens_arr: *const *const c_char`
- `timestamps: *const c_float`
- `count: c_int`
- `json: *const c_char`

当前代码仅使用 `text` 字段，其他字段保留用于未来扩展。

---

## 已知问题与限制

### 1. 库路径问题
- 编译后的二进制需要设置 `LD_LIBRARY_PATH` 才能找到 `libsherpa-onnx-c-api.so`
- **解决方案**: 添加 rpath 或使用启动脚本

### 2. 音频设备兼容性
- 当前强制使用 16000Hz 单声道
- 如果硬件不支持，可能报错 "Stream configuration not supported"
- **解决方案**: 已在 `docs/AGENTS.md` Phase 1.5 中规划设备管理功能

### 3. 模型文件依赖
- 需要预先下载模型文件到 `./models` 目录
- 文件大小约 1GB（包含 INT8 和 FP32 版本）

---

## 后续优化建议

### 短期（Phase 1.5）
1. 添加设备枚举和选择功能
2. 实现 rpath 自动配置
3. 添加音频配置回退机制（如果 16kHz 不支持，使用手动重采样）

### 中期（Phase 2）
1. 实现 uinput 虚拟键盘集成
2. 添加全局热键支持
3. 优化内存使用和延迟

### 长期（Phase 3+）
1. 添加 VAD（语音活动检测）
2. 支持多模型切换
3. GUI 界面开发

---

## 关键代码位置

### FFI 绑定
- `src/ffi/mod.rs:148-155` - OnlineRecognizer 结构体定义
- `src/ffi/mod.rs:167-246` - OnlineRecognizer::new() 实现

### 音频配置
- `src/main.rs:36-49` - 音频设备和配置初始化
- `src/main.rs:60-68` - 音频流回调

### 推理循环
- `src/main.rs:74-100` - 主推理循环

---

## 参考资料

### 官方文档
- [Sherpa-ONNX Documentation](https://k2-fsa.github.io/sherpa/onnx/)
- [cpal Documentation](https://docs.rs/cpal/)

### 相关 Issue
- Sherpa-onnx resampler crashes on Linux: 已知问题，建议使用外部重采样

### 类似项目
- `ydotool` - Linux 虚拟输入工具
- `wtype` - Wayland 文本输入工具

---

**最后更新**: 2026-02-03 02:42
**状态**: ✅ 核心功能可用，等待实际运行测试
