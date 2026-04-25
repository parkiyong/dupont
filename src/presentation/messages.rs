use crate::application::dto::SettingsDto;
use crate::domain::value_objects::SourceId;

#[derive(Debug, Clone)]
pub enum AppMsg {
    Refresh,
    SourceChanged(SourceId),
    SettingsChanged(SettingsDto),
    WallpaperLoaded {
        id: String,
        title: String,
        description: String,
        attribution: String,
        source: String,
        cache_path: std::path::PathBuf,
    },
    FetchError(String),
}
