# Cinnabar 文档中心

欢迎来到 Cinnabar（朱砂）项目的文档中心。本目录包含了项目的所有技术文档和用户指南。

## 📚 文档导航

### 核心文档

- **[开发路线图](ROADMAP.md)** - 项目规划、版本历史、FAQ 和已知问题
- **[架构设计文档](../AGENTS.md)** - 详细的技术架构、设计决策和实现细节

### 用户指南

- **[安装指南](INSTALL.md)** - 详细的安装步骤、平台支持和权限配置
- **[故障排除指南](TROUBLESHOOTING.md)** - 常见问题诊断和解决方案
- **[性能调优指南](PERFORMANCE.md)** - 性能优化建议、基准测试和调优技巧

### 开发者指南

- **[贡献指南](CONTRIBUTING.md)** - 如何为项目做出贡献、代码规范和提交流程

## 🚀 快速开始

### 新用户

1. 阅读 [安装指南](INSTALL.md) 了解如何安装 Cinnabar
2. 查看 [主 README](../README.md) 了解项目概述和基本使用
3. 遇到问题时参考 [故障排除指南](TROUBLESHOOTING.md)

### 开发者

1. 阅读 [架构设计文档](../AGENTS.md) 了解项目架构
2. 查看 [开发路线图](ROADMAP.md) 了解项目规划
3. 参考 [贡献指南](CONTRIBUTING.md) 开始贡献代码

### 性能优化

1. 查看 [性能调优指南](PERFORMANCE.md) 了解性能基准
2. 根据硬件配置调整参数
3. 使用基准测试工具验证优化效果

## 📖 文档结构

```
docs/
├── README.md              # 本文件 - 文档索引
├── ROADMAP.md             # 开发路线图和 FAQ
├── INSTALL.md             # 安装指南
├── TROUBLESHOOTING.md     # 故障排除指南
├── PERFORMANCE.md         # 性能调优指南
└── CONTRIBUTING.md        # 贡献指南
```

## 🔍 按主题查找

### 安装相关
- [系统要求](INSTALL.md#系统要求)
- [快速安装](INSTALL.md#快速安装)
- [分发行版安装](INSTALL.md#分发行版安装)
- [权限配置](INSTALL.md#权限配置)

### 使用相关
- [CLI 模式使用](../README.md#使用方法)
- [GUI 模式使用](ROADMAP.md#使用指南)
- [配置文件](../README.md#配置文件)
- [设备选择](TROUBLESHOOTING.md#运行时问题)

### 问题排查
- [编译问题](TROUBLESHOOTING.md#编译问题)
- [运行时问题](TROUBLESHOOTING.md#运行时问题)
- [音频问题](TROUBLESHOOTING.md#音频问题)
- [识别问题](TROUBLESHOOTING.md#识别问题)

### 性能优化
- [性能基准](PERFORMANCE.md#性能基准)
- [CPU 优化](PERFORMANCE.md#cpu-优化)
- [内存优化](PERFORMANCE.md#内存优化)
- [延迟优化](PERFORMANCE.md#延迟优化)

### 开发相关
- [架构设计](../AGENTS.md#architecture)
- [代码规范](CONTRIBUTING.md#代码规范)
- [测试要求](CONTRIBUTING.md#测试要求)
- [提交规范](CONTRIBUTING.md#提交规范)

## 🆘 获取帮助

### 常见问题

查看 [开发路线图 - FAQ](ROADMAP.md#常见问题faq) 获取常见问题的答案。

### 故障排除

遇到问题时，请按以下顺序查找解决方案：

1. 查看 [故障排除指南](TROUBLESHOOTING.md)
2. 搜索 [GitHub Issues](https://github.com/yourusername/cinnabar/issues)
3. 在 [GitHub Discussions](https://github.com/yourusername/cinnabar/discussions) 提问
4. 联系维护者

### 报告问题

提交 Issue 时请包含：

- 清晰的问题描述
- 重现步骤
- 系统信息（OS、Rust 版本、音频栈）
- 相关日志（使用 `--verbose` 模式）

参考 [贡献指南 - 报告 Bug](CONTRIBUTING.md#报告-bug)

## 📝 文档贡献

发现文档错误或需要改进？欢迎贡献！

1. Fork 本仓库
2. 修改文档
3. 提交 Pull Request

详见 [贡献指南 - 文档贡献](CONTRIBUTING.md#文档贡献)

## 🔗 外部资源

- [Sherpa-ONNX 文档](https://k2-fsa.github.io/sherpa/onnx/)
- [Paraformer 论文](https://arxiv.org/abs/2206.08317)
- [cpal 文档](https://docs.rs/cpal/)
- [Rust 官方文档](https://doc.rust-lang.org/)

## 📊 文档版本

- **当前版本**: v1.2.3
- **最后更新**: 2026-02-03
- **维护者**: Cinnabar Team

---

**提示**: 所有文档均使用 Markdown 格式编写，可以在 GitHub 上直接阅读，也可以使用任何 Markdown 阅读器查看。