# Cinnabar GUI 模式设计文档

## 概述

Cinnabar 将支持两种运行模式：
1. **CLI 模式**：纯终端模式，用于研究和测试语音识别效果
2. **GUI 模式**：悬浮窗模式，用于日常语音输入

## 架构设计

### 模式切换

```cinnabar/src/main.rs#L1-30
#[derive(Parser, Debug)]
struct Args {
    /// 运行模式：cli 或 gui
    #[arg(short, long, default_value = "cli")]
    mode: String,
    
    // ... 其他参数
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    match args.mode.as_str() {
        "cli" => run_cli_mode(args),
        "gui" => run_gui_mode(args),
        _ => bail!("Invalid mode. Use 'cli' or 'gui'"),
    }
}
```

---

## CLI 模式（当前实现）

### 功能
- 实时显示识别结果
- 基于标点符号的句子分割
- 终端输出
- 调试模式（--verbose）

### 使用场景
- 测试语音识别准确率
- 调试音频配置
- 研究模型性能

### 命令示例
```cinnabar/docs/GUI_MODE_DESIGN.md#L1-100
# 默认 CLI 模式
cargo run --release

# 显式指定 CLI 模式
cargo run --release -- --mode cli

# 启用调试输出
cargo run --release -- --mode cli --verbose
```

---

## GUI 模式（新增）

### 核心功能

#### 1. 悬浮窗
- 小型透明窗口
- 显示当前状态（待机/监听/识别中）
- 显示识别结果预览
- 自动定位到输入框附近

#### 2. 热键激活
- **F3**：触发语音输入
- 按下 F3 开始监听
- 再次按下 F3 或检测到句子结束时停止

#### 3. 自动文本注入
- 识别完成后自动注入到激活的文本框
- 使用剪贴板粘贴策略（Ctrl+V）
- 支持中文和英文

#### 4. 窗口定位
- 检测当前激活窗口
- 获取输入框位置
- 悬浮窗定位到输入框附近（右上角或下方）

### 技术栈

#### GUI 框架选择

**推荐：egui + winit**
- **egui**：轻量级即时模式 GUI
- **winit**：跨平台窗口管理，Wayland 支持良好
- **优点**：
  - 原生 Wayland 支持
  - 轻量级（~2MB 二进制）
  - 易于实现悬浮窗
  - 即时模式，状态管理简单

**备选：iced**
- Elm 风格声明式 GUI
- Wayland 支持
- 更复杂但更强大

#### 热键监听

**global-hotkey (0.6)**
- 跨平台全局热键
- Wayland 支持
- 简单 API

#### 窗口管理

**Wayland 协议**
- `wlr-foreign-toplevel-management`：获取窗口列表
- `zwlr_layer_shell_v1`：创建悬浮层窗口
- 通过 `wayland-client` crate 实现

### 依赖更新

```cinnabar/Cargo.toml#L1-30
[dependencies]
# 现有依赖
cpal = "0.15"
anyhow = "1.0"
clap = { version = "4.5", features = ["derive"] }
sherpa-rs-sys = "0.6"
crossbeam-channel = "0.5"
ctrlc = "3.4"
arboard = "3.4"
evdev = "0.12"

# GUI 模式新增依赖
egui = "0.30"
eframe = { version = "0.30", default-features = false, features = ["wayland"] }
global-hotkey = "0.6"
wayland-client = "0.31"
```

---

## 实现计划

### Phase 1: 基础 GUI 框架（v1.1.0）

**目标**：实现基本的悬浮窗和模式切换

**任务**：
- [ ] 创建 `src/gui/mod.rs` 模块
- [ ] 实现基础悬浮窗（egui + eframe）
- [ ] 实现模式切换逻辑
- [ ] 显示简单的状态信息

**文件结构**：
```cinnabar/docs/GUI_MODE_DESIGN.md#L1-200
src/
├── main.rs          # 模式切换入口
├── cli.rs           # CLI 模式实现（重构现有代码）
├── gui/
│   ├── mod.rs       # GUI 模式入口
│   ├── window.rs    # 悬浮窗实现
│   └── state.rs     # GUI 状态管理
├── ffi/
├── injector.rs
└── resampler.rs
```

**预期效果**：
```cinnabar/docs/GUI_MODE_DESIGN.md#L1-200
# 运行 GUI 模式
cargo run --release -- --mode gui

# 显示一个小型悬浮窗
# 窗口内容：
# ┌─────────────────┐
# │ Cinnabar        │
# │ 状态: 待机      │
# │ 按 F3 开始      │
# └─────────────────┘
```

### Phase 2: 热键集成（v1.2.0）

**目标**：实现 F3 热键触发语音输入

**任务**：
- [ ] 集成 `global-hotkey` crate
- [ ] 注册 F3 热键
- [ ] 实现热键回调
- [ ] 连接语音识别模块

**状态流转**：
```cinnabar/docs/GUI_MODE_DESIGN.md#L1-200
待机 --[按下 F3]--> 监听中 --[识别到文本]--> 识别中 --[句子结束]--> 注入文本 --> 待机
```

### Phase 3: 文本注入集成（v1.3.0）

**目标**：自动注入识别结果到激活窗口

**任务**：
- [ ] 集成 `TextInjector` 模块
- [ ] 实现自动注入逻辑
- [ ] 添加注入成功/失败反馈

### Phase 4: 窗口定位（v1.4.0）

**目标**：悬浮窗自动定位到输入框附近

**任务**：
- [ ] 实现 Wayland 窗口信息获取
- [ ] 检测激活窗口位置
- [ ] 计算悬浮窗位置
- [ ] 实现自动定位

**定位策略**：
```cinnabar/docs/GUI_MODE_DESIGN.md#L1-200
┌─────────────────────────────┐
│ 激活窗口                    │
│                             │
│  ┌──────────────┐           │
│  │ 输入框       │  ┌──────┐ │
│  └──────────────┘  │Cinna-│ │ <- 悬浮窗定位到输入框右上角
│                    │bar   │ │
│                    └──────┘ │
└─────────────────────────────┘
```

---

## GUI 界面设计

### 待机状态
```cinnabar/docs/GUI_MODE_DESIGN.md#L1-300
┌─────────────────┐
│ 🎤 Cinnabar     │
│ ─────────────── │
│ 状态: 待机      │
│ 按 F3 开始      │
└─────────────────┘
```

### 监听状态
```cinnabar/docs/GUI_MODE_DESIGN.md#L1-300
┌─────────────────┐
│ 🔴 Cinnabar     │
│ ─────────────── │
│ 状态: 监听中... │
│ 按 F3 停止      │
└─────────────────┘
```

### 识别状态
```cinnabar/docs/GUI_MODE_DESIGN.md#L1-300
┌─────────────────┐
│ 🟢 Cinnabar     │
│ ─────────────── │
│ 我觉得 Rust...  │
│ 识别中          │
└─────────────────┘
```

### 注入状态
```cinnabar/docs/GUI_MODE_DESIGN.md#L1-300
┌─────────────────┐
│ ✅ Cinnabar     │
│ ─────────────── │
│ 已注入文本      │
│ 我觉得 Rust 很强│
└─────────────────┘
```

---

## 配置选项

### GUI 模式配置

```cinnabar/src/main.rs#L1-50
#[derive(Parser, Debug)]
struct Args {
    /// 运行模式
    #[arg(short, long, default_value = "cli")]
    mode: String,
    
    /// GUI 窗口透明度 (0.0-1.0)
    #[arg(long, default_value = "0.9")]
    opacity: f32,
    
    /// 悬浮窗位置：auto, top-right, bottom-right
    #[arg(long, default_value = "auto")]
    position: String,
    
    /// 热键（默认 F3）
    #[arg(long, default_value = "F3")]
    hotkey: String,
    
    // ... 其他参数
}
```

### 使用示例

```cinnabar/docs/GUI_MODE_DESIGN.md#L1-300
# 启动 GUI 模式
cargo run --release -- --mode gui

# 自定义透明度
cargo run --release -- --mode gui --opacity 0.8

# 自定义热键
cargo run --release -- --mode gui --hotkey F4

# 固定窗口位置
cargo run --release -- --mode gui --position top-right
```

---

## Wayland 兼容性

### 支持的合成器
- ✅ Sway
- ✅ Hyprland
- ✅ GNOME (Mutter)
- ✅ KDE Plasma (KWin)
- ⚠️ wlroots-based compositors（需要测试）

### 所需协议
- `wl_compositor` - 基础窗口创建
- `xdg_wm_base` - 窗口管理
- `zwlr_layer_shell_v1` - 悬浮层（可选）
- `wlr_foreign_toplevel_management` - 窗口信息（可选）

### 降级策略
如果高级协议不可用：
1. 使用普通窗口代替悬浮层
2. 固定窗口位置而非自动定位
3. 提示用户手动定位窗口

---

## 性能考虑

### 资源占用
- **内存**：~50MB（GUI 模式）vs ~20MB（CLI 模式）
- **CPU**：GUI 渲染 ~1-2% vs CLI ~0.5%
- **启动时间**：GUI ~200ms vs CLI ~50ms

### 优化策略
- 使用即时模式 GUI（egui）减少状态管理开销
- 仅在需要时渲染窗口
- 复用语音识别模块，避免重复初始化

---

## 测试计划

### 单元测试
- [ ] 模式切换逻辑
- [ ] 热键注册和回调
- [ ] 状态机转换

### 集成测试
- [ ] CLI 模式完整流程
- [ ] GUI 模式完整流程
- [ ] 文本注入功能

### 手动测试
- [ ] 不同 Wayland 合成器测试
- [ ] 不同应用程序测试（浏览器、编辑器、终端）
- [ ] 中英文混合输入测试
- [ ] 长时间运行稳定性测试

---

## 用户体验

### 典型工作流

**场景 1：在浏览器中输入**
1. 用户在浏览器中点击输入框
2. 悬浮窗自动出现在输入框附近
3. 用户按下 F3
4. 悬浮窗显示"监听中"
5. 用户说话："我觉得 Rust 很强大"
6. 悬浮窗显示识别结果
7. 检测到句号，自动注入文本
8. 悬浮窗显示"已注入"，2 秒后恢复待机

**场景 2：在终端中输入**
1. 用户在终端中准备输入命令
2. 按下 F3
3. 说话："cargo run release"
4. 自动注入到终端

### 错误处理

**权限错误**：
```cinnabar/docs/GUI_MODE_DESIGN.md#L1-400
⚠️ 无法创建虚拟键盘设备
请运行：sudo usermod -aG input $USER
然后重新登录
```

**热键冲突**：
```cinnabar/docs/GUI_MODE_DESIGN.md#L1-400
⚠️ F3 热键已被占用
请使用 --hotkey 参数指定其他热键
例如：--hotkey F4
```

**注入失败**：
```cinnabar/docs/GUI_MODE_DESIGN.md#L1-400
❌ 文本注入失败
已复制到剪贴板，请手动粘贴
```

---

## 未来增强

### v1.5.0+
- [ ] 多热键支持（F3 开始，F4 停止）
- [ ] 历史记录功能
- [ ] 撤销/重做
- [ ] 自定义词汇表
- [ ] 语音命令（如"删除"、"换行"）

### v2.0.0+
- [ ] 系统托盘集成
- [ ] 配置面板 GUI
- [ ] 多语言支持
- [ ] 云同步配置

---

## 开发优先级

### 高优先级（v1.1.0 - v1.3.0）
1. 基础 GUI 框架
2. 热键集成
3. 文本注入集成

### 中优先级（v1.4.0）
4. 窗口定位

### 低优先级（v1.5.0+）
5. 高级功能和优化

---

## 参考资源

### Wayland 协议
- [Wayland Protocol](https://wayland.freedesktop.org/docs/html/)
- [wlr-protocols](https://gitlab.freedesktop.org/wlroots/wlr-protocols)

### GUI 框架
- [egui](https://github.com/emilk/egui)
- [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)

### 热键库
- [global-hotkey](https://github.com/tauri-apps/global-hotkey)

---

**创建时间**: 2026-02-03 14:41  
**版本**: 1.0.0  
**状态**: 设计阶段  
**下一步**: 实现 Phase 1 - 基础 GUI 框架