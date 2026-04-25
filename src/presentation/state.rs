use crate::application::dto::{SettingsDto, WallpaperDto};
use crate::domain::value_objects::SourceId;

#[derive(Debug, Clone)]
pub struct AppState {
    pub current_wallpaper: Option<WallpaperDto>,
    pub loading: bool,
    pub settings: SettingsDto,
    pub selected_source: SourceId,
    pub error_message: Option<String>,
}

impl AppState {
    pub fn new(settings: SettingsDto) -> Self {
        Self {
            current_wallpaper: None,
            loading: false,
            settings,
            selected_source: SourceId::Bing,
            error_message: None,
        }
    }
}
