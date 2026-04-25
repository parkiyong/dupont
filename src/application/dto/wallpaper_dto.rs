use crate::domain::Wallpaper;

#[derive(Debug, Clone)]
pub struct WallpaperDto {
    pub id: String,
    pub title: String,
    pub description: String,
    pub attribution: String,
    pub source: String,
    pub cache_path: std::path::PathBuf,
}

impl From<(Wallpaper, std::path::PathBuf)> for WallpaperDto {
    fn from((wallpaper, cache_path): (Wallpaper, std::path::PathBuf)) -> Self {
        Self {
            id: wallpaper.id,
            title: wallpaper.title,
            description: wallpaper.description,
            attribution: wallpaper.attribution,
            source: wallpaper.source,
            cache_path,
        }
    }
}
