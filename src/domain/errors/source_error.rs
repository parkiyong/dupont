use thiserror::Error;

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Failed to parse API response: {0}")]
    ParseError(String),

    #[error("No wallpaper found in API response")]
    NoWallpaperFound,

    #[error("Source unavailable: {source_name}")]
    Unavailable { source_name: String },

    #[error("Rate limited by {source_name}")]
    RateLimited { source_name: String },
}
