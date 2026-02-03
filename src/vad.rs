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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vad_silence() {
        let vad = VadDetector::new(0.01);
        let silence = vec![0.0; 100];
        assert!(!vad.is_speech(&silence));
    }

    #[test]
    fn test_vad_speech() {
        let vad = VadDetector::new(0.01);
        let speech: Vec<f32> = (0..100).map(|i| (i as f32 * 0.1).sin()).collect();
        assert!(vad.is_speech(&speech));
    }

    #[test]
    fn test_vad_threshold() {
        let vad = VadDetector::new(0.5);
        let low_energy = vec![0.1; 100];
        assert!(!vad.is_speech(&low_energy));
    }
}
