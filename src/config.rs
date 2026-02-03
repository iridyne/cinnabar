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
