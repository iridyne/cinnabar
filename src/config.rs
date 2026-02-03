use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_model_dir")]
    pub model_dir: String,
    #[serde(default = "default_vad_threshold")]
    pub vad_threshold: f32,
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
}

fn default_model_dir() -> String {
    "./models".to_string()
}

fn default_vad_threshold() -> f32 {
    0.01
}

fn default_hotkey() -> String {
    "F3".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model_dir: default_model_dir(),
            vad_threshold: default_vad_threshold(),
            hotkey: default_hotkey(),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.model_dir, "./models");
        assert_eq!(config.vad_threshold, 0.01);
        assert_eq!(config.hotkey, "F3");
    }

    #[test]
    fn test_config_load_nonexistent() {
        let result = Config::load(&std::path::PathBuf::from("/nonexistent/config.toml"));
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.model_dir, "./models");
    }

    #[test]
    fn test_config_load_valid() {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file, "model_dir = \"/custom/models\"").unwrap();
        writeln!(temp_file, "vad_threshold = 0.05").unwrap();
        writeln!(temp_file, "hotkey = \"F4\"").unwrap();

        let config = Config::load(temp_file.path()).unwrap();
        assert_eq!(config.model_dir, "/custom/models");
        assert_eq!(config.vad_threshold, 0.05);
        assert_eq!(config.hotkey, "F4");
    }
}
