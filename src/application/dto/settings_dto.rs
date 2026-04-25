use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsDto {
    pub bing_market: String,
    pub spotlight_locale: String,
}

impl Default for SettingsDto {
    fn default() -> Self {
        Self {
            bing_market: "en-US".to_string(),
            spotlight_locale: "en-US".to_string(),
        }
    }
}
