use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Persisted user preferences.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub bing_market: String,
    pub spotlight_locale: String,
    pub active_source: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bing_market: "en-US".to_string(),
            spotlight_locale: "en-US".to_string(),
            active_source: "bing".to_string(),
        }
    }
}

impl Config {
    /// Returns the path to the config file: `$XDG_CONFIG_HOME/dupont/config.json`.
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("dupont")
            .join("config.json")
    }

    /// Load config from disk. Returns defaults if file is missing or corrupt.
    pub fn load() -> Self {
        let path = Self::config_path();
        if !path.exists() {
            let config = Self::default();
            // Best-effort write of defaults
            let _ = config.save();
            return config;
        }

        let data = match fs::read_to_string(&path) {
            Ok(d) => d,
            Err(_) => {
                let config = Self::default();
                let _ = config.save();
                return config;
            }
        };

        match serde_json::from_str(&data) {
            Ok(config) => config,
            Err(_) => {
                let config = Self::default();
                let _ = config.save();
                config
            }
        }
    }

    /// Save config to disk, creating parent directories if needed.
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config dir: {}", parent.display()))?;
        }
        let data = serde_json::to_string_pretty(self)?;
        fs::write(&path, data)
            .with_context(|| format!("Failed to write config: {}", path.display()))?;
        Ok(())
    }
}
