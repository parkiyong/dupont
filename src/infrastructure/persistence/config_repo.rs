use crate::application::dto::SettingsDto;
use crate::application::ports::ConfigRepository;
use crate::application::ports::config_repository::ConfigError;
use std::fs;
use std::path::PathBuf;

pub struct ConfigRepoImpl {
    path: PathBuf,
}

impl ConfigRepoImpl {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl ConfigRepository for ConfigRepoImpl {
    fn load(&self) -> SettingsDto {
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

    fn save(&self, settings: &SettingsDto) -> Result<(), ConfigError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(settings)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))?;
        fs::write(&self.path, data)?;
        Ok(())
    }
}
