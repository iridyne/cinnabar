#!/bin/bash

# 测试脚本：验证 Paraformer 模型是否输出标点符号
# 用法：./test_punctuation.sh

echo "🔍 测试 Paraformer 模型标点输出"
echo "================================"
echo ""
echo "请说一句完整的话（例如：我觉得 Rust 很强大）"
echo "观察输出是否包含标点符号（。？！等）"
echo ""
echo "按 Ctrl+C 停止测试"
echo ""

# 运行程序并启用 verbose 模式以便观察详细输出
cargo run --release -- --verbose
