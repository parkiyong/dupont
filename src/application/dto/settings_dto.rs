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

pub static BING_MARKETS: &[&str] = &[
    "en-US", "zh-CN", "ja-JP", "en-AU", "en-GB", "de-DE", "en-NZ", "en-CA", "en-IN", "fr-FR",
    "fr-CA",
];

pub static SPOTLIGHT_LOCALES: &[&str] = &[
    "en-US", "zh-CN", "ja-JP", "en-AU", "en-GB", "de-DE", "en-NZ", "en-CA", "en-IN", "fr-FR",
    "fr-CA",
];

pub fn bing_market_label(code: &str) -> String {
    match code {
        "en-US" => "English (United States)".to_string(),
        "zh-CN" => "Chinese (Simplified)".to_string(),
        "ja-JP" => "Japanese (Japan)".to_string(),
        "en-AU" => "English (Australia)".to_string(),
        "en-GB" => "English (United Kingdom)".to_string(),
        "de-DE" => "German (Germany)".to_string(),
        "en-NZ" => "English (New Zealand)".to_string(),
        "en-CA" => "English (Canada)".to_string(),
        "en-IN" => "English (India)".to_string(),
        "fr-FR" => "French (France)".to_string(),
        "fr-CA" => "French (Canada)".to_string(),
        _ => code.to_string(),
    }
}

pub fn spotlight_locale_label(code: &str) -> String {
    match code {
        "en-US" => "English (United States)".to_string(),
        "zh-CN" => "Chinese (Simplified)".to_string(),
        "ja-JP" => "Japanese (Japan)".to_string(),
        "en-AU" => "English (Australia)".to_string(),
        "en-GB" => "English (United Kingdom)".to_string(),
        "de-DE" => "German (Germany)".to_string(),
        "en-NZ" => "English (New Zealand)".to_string(),
        "en-CA" => "English (Canada)".to_string(),
        "en-IN" => "English (India)".to_string(),
        "fr-FR" => "French (France)".to_string(),
        "fr-CA" => "French (Canada)".to_string(),
        _ => code.to_string(),
    }
}
