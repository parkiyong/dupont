use crate::application::dto::SettingsDto;

pub trait ConfigRepository: Send + Sync {
    fn load(&self) -> SettingsDto;
    fn save(&self, settings: &SettingsDto) -> Result<(), ConfigError>;
}

#[derive(Debug)]
pub enum ConfigError {
    IoError(std::io::Error),
    SerializationError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::SerializationError(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::IoError(err)
    }
}
