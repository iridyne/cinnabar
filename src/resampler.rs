pub struct LinearResampler {
    ratio: f32,
    buffer: Vec<f32>,
}

impl LinearResampler {
    pub fn new(input_rate: u32, output_rate: u32) -> Self {
        Self {
            ratio: input_rate as f32 / output_rate as f32,
            buffer: Vec::new(),
        }
    }

    pub fn resample(&mut self, input: &[f32]) -> Vec<f32> {
        if input.is_empty() {
            return Vec::new();
        }

        self.buffer.extend_from_slice(input);
        let output_len = (self.buffer.len() as f32 / self.ratio) as usize;
        let mut output = Vec::with_capacity(output_len);

        for i in 0..output_len {
            let pos = i as f32 * self.ratio;
            let idx = pos as usize;

            if idx + 1 < self.buffer.len() {
                let frac = pos - idx as f32;
                let sample = self.buffer[idx] * (1.0 - frac) + self.buffer[idx + 1] * frac;
                output.push(sample);
            }
        }

        let consumed = (output_len as f32 * self.ratio) as usize;
        self.buffer.drain(..consumed.min(self.buffer.len()));

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resampler_basic() {
        let mut resampler = LinearResampler::new(48000, 16000);
        let input: Vec<f32> = (0..480).map(|i| (i as f32 * 0.01).sin()).collect();
        let output = resampler.resample(&input);
        assert!(!output.is_empty());
        assert!(output.len() < input.len());
    }

    #[test]
    fn test_resampler_empty() {
        let mut resampler = LinearResampler::new(48000, 16000);
        let output = resampler.resample(&[]);
        assert!(output.is_empty());
    }

    #[test]
    fn test_resampler_ratio() {
        let mut resampler = LinearResampler::new(48000, 16000);
        let input = vec![1.0; 480];
        let output = resampler.resample(&input);
        assert_eq!(output.len(), 160);
    }
}
