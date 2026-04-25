use crate::application::dto::SettingsDto;
use std::fs;
use std::path::PathBuf;

pub struct ConfigRepo {
    path: PathBuf,
}

impl ConfigRepo {
    pub fn new() -> Self {
        Self {
            path: dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("dupont")
                .join("config.json"),
        }
    }

    pub fn load(&self) -> SettingsDto {
        if !self.path.exists() {
            let settings = SettingsDto::default();
            let _ = self.save(&settings);
            return settings;
        }

        let data = match fs::read_to_string(&self.path) {
            Ok(d) => d,
            Err(_) => {
                let settings = SettingsDto::default();
                let _ = self.save(&settings);
                return settings;
            }
        };

        serde_json::from_str(&data).unwrap_or_else(|_| {
            let settings = SettingsDto::default();
            let _ = self.save(&settings);
            settings
        })
    }

    pub fn save(&self, settings: &SettingsDto) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let data = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
        fs::write(&self.path, data).map_err(|e| e.to_string())?;
        Ok(())
    }
}

impl Default for ConfigRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ConfigRepo {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
        }
    }
}
