# Cinnabar 标点功能分析

## 当前状态

### Paraformer 模型标点能力

当前使用的模型：`sherpa-onnx-streaming-paraformer-bilingual-zh-en`

**测试结果**：需要实际测试验证
- 模型名称中未明确标注 "with-punc" 或 "no-punc"
- 代码中已实现基于标点符号的句子分割（检测 。？！.?!）
- 如果模型不输出标点，句子分割功能将无法正常工作

### 当前实现

```cinnabar/src/main.rs#L227-247
// 检测句子结束标点
let has_sentence_end = trimmed.ends_with('。')
    || trimmed.ends_with('？')
    || trimmed.ends_with('！')
    || trimmed.ends_with('.')
    || trimmed.ends_with('?')
    || trimmed.ends_with('!');

if has_sentence_end && trimmed != last_result {
    // 句子结束，输出完整句子并换行
    print!("\r\x1b[K{}\n", trimmed);
    std::io::Write::flush(&mut std::io::stdout()).ok();
    last_result.clear();
} else {
    // 句子未结束，在同一行更新
    print!("\r\x1b[K{}", trimmed);
    std::io::Write::flush(&mut std::io::stdout()).ok();
    last_result = trimmed.to_string();
}
```

---

## 方案分析

### 方案 A：验证 Paraformer 自带标点

**优点**：
- 无需额外模型
- 零延迟
- 零额外 CPU 开销
- 代码已实现

**缺点**：
- 需要验证模型是否真的输出标点
- 如果不输出标点，需要切换方案

**实施步骤**：
1. 运行程序并说话测试
2. 观察输出是否包含标点符号
3. 如果有标点，方案完成
4. 如果无标点，切换到方案 B 或 C

---

### 方案 B：集成 CT-Transformer 标点模型

**模型信息**：
- 名称：`sherpa-onnx-punc-ct-transformer-zh-en-common-2023-02-08`
- 大小：约 50MB
- 语言：中英通用
- 性能：CPU 毫秒级

**优点**：
- 专业的标点恢复模型
- 支持中英文
- 轻量级，CPU 可运行

**缺点**：
- 增加模型下载和管理复杂度
- 引入延迟（需要看后续文本才能决定标点）
- Rust FFI 绑定可能不完善
- 流式场景下延迟问题明显

**技术挑战**：
1. **FFI 绑定问题**：sherpa-rs 可能不支持 OnlinePunctuation API
2. **延迟问题**：标点模型需要上下文，会引入延迟
3. **流式集成**：如何在流式输出中平滑集成标点

**实施步骤**：
1. 下载标点模型
2. 检查 sherpa-rs FFI 是否支持 OnlinePunctuation
3. 如果不支持，需要手动添加 FFI 绑定
4. 集成到主循环，处理延迟问题
5. 测试和优化

---

### 方案 C：基于规则的标点模拟（推荐）

**规则设计**：
- 停顿 > 500ms：添加逗号 `,`
- 停顿 > 1000ms：添加句号 `。`
- 语气词检测：`吗` → `？`，`吧` → `。`
- 感叹词检测：`啊`、`哇` → `！`

**优点**：
- 实现简单，约 50-100 行代码
- 零延迟
- 零额外模型
- 性价比最高
- 适合轻量化项目定位

**缺点**：
- 准确率不如专业模型
- 规则需要调优
- 可能出现误判

**实施步骤**：
1. 记录每次识别结果的时间戳
2. 计算相邻结果的时间间隔
3. 根据间隔添加标点
4. 实现语气词检测规则
5. 测试和调优

**代码示例**：
```cinnabar/src/main.rs#L1-50
use std::time::Instant;

let mut last_update_time = Instant::now();
let mut accumulated_text = String::new();

// 在主循环中
let current_time = Instant::now();
let silence_duration = current_time.duration_since(last_update_time);

if silence_duration.as_millis() > 500 {
    // 添加逗号
    accumulated_text.push('，');
} else if silence_duration.as_millis() > 1000 {
    // 添加句号
    accumulated_text.push('。');
}

// 语气词检测
if accumulated_text.ends_with("吗") {
    accumulated_text.push('？');
}

last_update_time = current_time;
```

---

## 推荐方案

### 短期（v1.1.0）：方案 A + 方案 C

1. **首先验证 Paraformer 是否自带标点**
   - 如果有标点，保持当前实现
   - 如果无标点，实施方案 C

2. **实现基于规则的标点模拟**
   - 停顿检测
   - 语气词检测
   - 简单高效

### 中期（v1.2.0）：方案 B（可选）

如果用户对标点准确率有更高要求：
1. 集成 CT-Transformer 标点模型
2. 作为可选功能（`--enable-punctuation-model`）
3. 用户可以选择使用规则或模型

---

## 测试计划

### 测试 1：验证 Paraformer 标点输出

```cinnabar/test_punctuation.sh#L1-16
#!/bin/bash
echo "🔍 测试 Paraformer 模型标点输出"
echo "请说：我觉得 Rust 很强大"
echo "观察输出是否包含标点符号"
cargo run --release
```

**预期结果**：
- 如果输出 `我觉得 Rust 很强大。`，说明模型自带标点
- 如果输出 `我觉得 Rust 很强大`，说明模型不带标点

### 测试 2：规则标点效果

测试停顿检测：
- 说话：`我觉得` [停顿 600ms] `Rust 很强大`
- 预期：`我觉得，Rust 很强大`

测试语气词：
- 说话：`你喜欢 Rust 吗`
- 预期：`你喜欢 Rust 吗？`

---

## 实施优先级

1. **高优先级**：验证 Paraformer 标点能力（5 分钟）
2. **中优先级**：实现基于规则的标点模拟（1-2 小时）
3. **低优先级**：集成 CT-Transformer 模型（4-8 小时）

---

## 结论

**推荐路径**：
1. 先测试 Paraformer 是否自带标点
2. 如果有标点，保持当前实现
3. 如果无标点，实现基于规则的标点模拟（方案 C）
4. 将 CT-Transformer 集成作为未来可选功能

**理由**：
- 符合项目"轻量化"定位
- 性价比最高
- 实现简单
- 用户体验良好

---

**创建时间**: 2026-02-03 12:55  
**状态**: 待测试验证  
**下一步**: 运行 `./test_punctuation.sh` 验证 Paraformer 标点能力