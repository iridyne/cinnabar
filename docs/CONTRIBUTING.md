# Cinnabar 贡献指南

感谢您对 Cinnabar 项目的关注！本文档将帮助您了解如何为项目做出贡献。

## 目录

- [行为准则](#行为准则)
- [如何贡献](#如何贡献)
- [开发环境设置](#开发环境设置)
- [代码规范](#代码规范)
- [提交规范](#提交规范)
- [测试要求](#测试要求)
- [文档贡献](#文档贡献)

---

## 行为准则

我们致力于为所有贡献者提供友好、安全和包容的环境。参与本项目即表示您同意遵守以下准则：

- 尊重不同的观点和经验
- 接受建设性的批评
- 关注对社区最有利的事情
- 对其他社区成员表示同理心

## 如何贡献

### 报告 Bug

如果您发现了 Bug，请：

1. 检查 [Issues](https://github.com/yourusername/cinnabar/issues) 确认问题未被报告
2. 创建新 Issue，包含：
   - 清晰的标题和描述
   - 重现步骤
   - 预期行为和实际行为
   - 系统信息（OS、Rust 版本、音频栈）
   - 相关日志（使用 `--verbose` 模式）

### 提出新功能

1. 先在 Issues 中讨论您的想法
2. 等待维护者反馈
3. 获得批准后再开始实现

### 提交代码

1. Fork 本仓库
2. 创建特性分支：`git checkout -b feature/your-feature`
3. 进行更改并提交
4. 推送到您的 Fork：`git push origin feature/your-feature`
5. 创建 Pull Request

---

## 开发环境设置

### 系统要求

- **操作系统**：Linux（推荐 Arch/Ubuntu）
- **Rust**：1.70+ (2021 edition)
- **CMake**：3.20+
- **音频**：ALSA 或 PipeWire

### 安装步骤

```bash
# 1. 克隆仓库
git clone https://github.com/yourusername/cinnabar.git
cd cinnabar

# 2. 下载模型
./setup_models.sh

# 3. 构建项目
cargo build --release

# 4. 运行测试
cargo test --release

# 5. 运行 clippy
cargo clippy --release

# 6. 格式化代码
cargo fmt
```

### 开发工具

推荐安装以下工具：

```bash
# Rust 工具链
rustup component add rustfmt clippy

# 性能分析
cargo install flamegraph
cargo install heaptrack

# 代码覆盖率
cargo install cargo-tarpaulin
```

---

## 代码规范

### Rust 代码风格

- 遵循 [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- 使用 `cargo fmt` 格式化代码
- 通过 `cargo clippy` 检查
- 避免 `unwrap()`，使用 `?` 或 `context()`

### 命名约定

- **变量**：`snake_case`
- **函数**：`snake_case`
- **类型**：`PascalCase`
- **常量**：`SCREAMING_SNAKE_CASE`
- **模块**：`snake_case`

### 注释规范

```rust
/// 函数文档注释（三斜杠）
///
/// # Arguments
/// * `param` - 参数描述
///
/// # Returns
/// 返回值描述
///
/// # Errors
/// 错误情况描述
pub fn example(param: i32) -> Result<String> {
    // 行内注释（双斜杠）
    Ok(format!("Result: {}", param))
}
```

### 错误处理

```rust
use anyhow::{Context, Result};

// ✅ 推荐
fn load_model(path: &Path) -> Result<Model> {
    let data = std::fs::read(path)
        .context("Failed to read model file")?;
    Ok(Model::from_bytes(&data)?)
}

// ❌ 避免
fn load_model(path: &Path) -> Model {
    let data = std::fs::read(path).unwrap();
    Model::from_bytes(&data).unwrap()
}
```

---

## 提交规范

### Commit Message 格式

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Type 类型

- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建/工具链更新

### 示例

```
feat(audio): 添加设备选择功能

- 实现设备枚举
- 添加 --list-devices 参数
- 支持按索引或名称选择设备

Closes #42
```

---

## 测试要求

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resampler_basic() {
        let mut resampler = LinearResampler::new(48000, 16000);
        let input = vec![1.0; 480];
        let output = resampler.resample(&input);
        assert_eq!(output.len(), 160);
    }
}
```

### 运行测试

```bash
# 运行所有测试
cargo test --release

# 运行特定测试
cargo test --release test_resampler

# 显示输出
cargo test --release -- --nocapture
```

### 测试覆盖率

```bash
# 生成覆盖率报告
cargo tarpaulin --out Html --output-dir coverage
```

---

## 文档贡献

### 文档类型

- **代码文档**：使用 `///` 注释
- **架构文档**：更新 `AGENTS.md`
- **用户文档**：更新 `README.md`
- **开发文档**：更新 `docs/` 目录

### 文档规范

- 使用清晰、简洁的语言
- 提供代码示例
- 包含使用场景
- 保持中英文一致性

### 生成文档

```bash
# 生成 API 文档
cargo doc --open

# 检查文档链接
cargo doc --no-deps
```

---

## Pull Request 流程

### 提交前检查

- [ ] 代码通过 `cargo fmt` 格式化
- [ ] 代码通过 `cargo clippy` 检查
- [ ] 所有测试通过 `cargo test --release`
- [ ] 添加了必要的测试
- [ ] 更新了相关文档
- [ ] Commit message 符合规范

### PR 描述模板

```markdown
## 变更类型
- [ ] Bug 修复
- [ ] 新功能
- [ ] 重构
- [ ] 文档更新

## 变更描述
简要描述您的更改...

## 测试
描述如何测试您的更改...

## 相关 Issue
Closes #123
```

### 代码审查

- 维护者会在 1-3 个工作日内审查
- 根据反馈进行修改
- 所有讨论解决后合并

---

## 优先级指南

### 高优先级

- Bug 修复
- 性能优化
- 安全问题
- 文档改进

### 中优先级

- 新功能实现
- 代码重构
- 测试覆盖率提升

### 低优先级

- 代码风格调整
- 依赖更新
- 工具链改进

---

## 获取帮助

如果您有任何问题：

1. 查看 [文档](../README.md)
2. 搜索 [Issues](https://github.com/yourusername/cinnabar/issues)
3. 在 [Discussions](https://github.com/yourusername/cinnabar/discussions) 提问
4. 联系维护者

---

## 许可证

贡献的代码将采用 MIT 许可证。提交 PR 即表示您同意此许可。

---

**感谢您的贡献！**

**最后更新**：2026-02-03  
**版本**：v1.2.3