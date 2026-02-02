# Cinnabar (朱砂)

[English](README_EN.md) | 简体中文

> 轻量级、离线优先的 Linux 流式语音转文字输入工具

## 项目简介

**Cinnabar（朱砂）** 是一个专为 Linux 系统设计的离线语音输入工具，使用 Rust 构建，基于 Sherpa-ONNX 推理引擎和阿里巴巴 Paraformer 模型实现实时流式语音识别。

### 核心特性

- 🔒 **完全离线** - 所有处理在本地完成，无需联网，保护隐私
- ⚡ **实时流式** - 边说边显示，支持部分结果和自动断句
- 🪶 **轻量高效** - 最小化依赖，优化边缘设备性能
- 🐧 **Linux 原生** - 深度集成 Linux 音频栈（ALSA/PipeWire）
- 🇨🇳 **中英双语** - 基于 Paraformer 双语模型，支持中英文混合识别

## 快速开始

### 系统要求

- **操作系统**: Linux（推荐 Arch/CachyOS）
- **Rust**: 1.70+ (2021 edition)
- **音频**: ALSA 或 PipeWire
- **CPU**: 支持 x86_64 架构

### 安装步骤

1. **克隆项目**
```bash
git clone https://github.com/yourusername/cinnabar.git
cd cinnabar
```

2. **下载模型**
```bash
./setup_models.sh
```

3. **编译运行**
```bash
cargo run --release
```

### 使用方法

#### 基本使用

```bash
# 使用默认模型目录 (./models)
cargo run --release

# 指定自定义模型目录
cargo run --release -- --model-dir /path/to/models
```

#### 预期输出

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

## 技术架构

### 核心组件

```
┌─────────────────┐         ┌──────────────────┐
│  音频捕获线程    │         │   推理线程        │
│                 │         │                  │
│  1. 麦克风采集   │         │  1. 接收音频      │
│  2. 单声道混音   │ ──────> │  2. 模型推理      │
│  3. 重采样 16kHz │ Channel │  3. 显示结果      │
│  4. 发送数据     │         │  4. 断句检测      │
└─────────────────┘         └──────────────────┘
```

### 关键技术

#### 音频重采样

硬件麦克风通常输出 44100Hz 或 48000Hz 采样率，但 Paraformer 模型严格要求 16000Hz。Cinnabar 实现了 `LinearResampler` 进行实时线性插值重采样：

- **算法**: 相邻采样点线性插值
- **状态管理**: 维护缓冲区处理跨块分数采样位置
- **性能**: 单次处理，最小内存开销

#### Actor 并发模型

使用 `crossbeam-channel` 实现类 Actor 架构：
- **音频线程**: 捕获 → 重采样 → 发送
- **推理线程**: 接收 → 解码 → 显示
- **有界通道**: 容量 100，防止内存无限增长

#### 自动断句

通过 `enable_endpoint: true` 启用模型内置的静音检测：
- 说话时显示部分结果（`🤔 Thinking: ...`）
- 检测到句子结束时显示最终结果（`✅ Final: ...`）
- 自动重置识别器状态，准备下一句

### 依赖项

| 依赖 | 用途 | 版本 |
|------|------|------|
| `cpal` | 跨平台音频 I/O | 0.15 |
| `sherpa-onnx` | ONNX 推理引擎 | 1.10 |
| `crossbeam-channel` | 无锁 MPSC 通道 | 0.5 |
| `anyhow` | 错误处理 | 1.0 |
| `clap` | CLI 参数解析 | 4.5 |
| `ctrlc` | 信号处理 | 3.4 |

## 性能指标

- **延迟**: 60-110ms 端到端（取决于 CPU）
- **内存**: ~100MB 运行时（包含音频缓冲）
- **模型大小**: ~40MB（INT8 量化）
- **CPU**: 单核可运行（现代 CPU）

## 开发路线图

### Phase 1: CLI 核心演示 ✅
- [x] 音频捕获与重采样
- [x] 流式语音识别
- [x] 自动断句
- [x] 终端输出

### Phase 2: 虚拟键盘集成
- [ ] Linux `uinput` 集成
- [ ] 模拟键盘事件
- [ ] 热键激活（如 Ctrl+Space）

### Phase 3: 高级特性
- [ ] 语音活动检测（VAD）
- [ ] 多模型支持（Whisper、Conformer）
- [ ] 标点符号恢复
- [ ] 自定义词汇注入

### Phase 4: 图形界面
- [ ] 系统托盘指示器
- [ ] 可视化监听状态
- [ ] 配置面板
- [ ] 模型管理界面

## 文档

- [架构设计文档](docs/AGENTS.md) - 详细的技术架构和设计决策

## 贡献指南

欢迎贡献！请遵循以下步骤：

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

## 致谢

- [Sherpa-ONNX](https://k2-fsa.github.io/sherpa/onnx/) - 高性能 ONNX 推理引擎
- [Alibaba Paraformer](https://arxiv.org/abs/2206.08317) - 优秀的中文语音识别模型
- [cpal](https://github.com/RustAudio/cpal) - Rust 跨平台音频库

---

**版本**: 0.1.0 (MVP)  
**状态**: 核心演示完成  
**最后更新**: 2026-02-03