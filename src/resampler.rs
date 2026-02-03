pub struct LinearResampler {
    from_rate: u32,
    to_rate: u32,
    buffer: Vec<f32>,
}

impl LinearResampler {
    pub fn new(from_rate: u32, to_rate: u32) -> Self {
        Self {
            from_rate,
            to_rate,
            buffer: Vec::new(),
        }
    }

    pub fn resample(&mut self, input: &[f32]) -> Vec<f32> {
        // 处理空输入
        if input.is_empty() {
            return Vec::new();
        }

        if self.from_rate == self.to_rate {
            return input.to_vec();
        }

        self.buffer.extend_from_slice(input);

        let ratio = self.from_rate as f64 / self.to_rate as f64;
        // 确保至少有 2 个样本才能进行插值
        if self.buffer.len() < 2 {
            return Vec::new();
        }

        let output_len = (self.buffer.len() as f64 / ratio).floor() as usize;

        if output_len == 0 {
            return Vec::new();
        }

        let mut output = Vec::with_capacity(output_len);

        for i in 0..output_len {
            let src_idx = i as f64 * ratio;
            let idx0 = src_idx as usize;
            let idx1 = idx0 + 1;

            if idx1 >= self.buffer.len() {
                break;
            }

            let frac = src_idx - idx0 as f64;
            let sample = self.buffer[idx0] * (1.0 - frac as f32) + self.buffer[idx1] * frac as f32;
            output.push(sample);
        }

        // 计算实际消耗的样本数，保留最后一个样本用于下次插值
        let consumed = if output.len() > 0 {
            let last_src_idx = ((output.len() - 1) as f64 * ratio) as usize;
            last_src_idx
        } else {
            0
        };

        // 清理已消耗的样本，但保留至少一个样本用于连续性
        if consumed > 0 && consumed < self.buffer.len() {
            self.buffer.drain(..consumed);
        } else if self.buffer.len() > 100 {
            // 防止缓冲区无限增长，如果超过阈值则清理
            let keep = self.buffer.len().saturating_sub(50);
            self.buffer.drain(..keep);
        }

        output
    }
}
