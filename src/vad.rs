pub struct VadDetector {
    threshold: f32,
}

impl VadDetector {
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }

    pub fn is_speech(&self, samples: &[f32]) -> bool {
        let energy: f32 = samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32;
        energy > self.threshold
    }
}
