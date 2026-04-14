use thiserror::Error;

/// Errors from wallpaper source fetching
#[derive(Debug, Error)]
pub enum SourceError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Failed to parse API response: {0}")]
    ParseError(String),

    #[error("No wallpaper found in API response")]
    NoWallpaperFound,

    #[error("Source unavailable: {source}")]
    Unavailable { source: String },

    #[error("Rate limited by {source}")]
    RateLimited { source: String },
}

/// Errors from cache operations
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Image decode error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("Cache directory not accessible")]
    NotAccessible,

    #[error("Cache size limit exceeded: {size}MB > {limit}MB")]
    SizeLimitExceeded { size: u64, limit: u64 },

    #[error("Cache item not found")]
    NotFound,
}

/// Errors from desktop environment operations
#[derive(Debug, Error)]
pub enum DEError {
    #[error("Failed to set wallpaper: {0}")]
    SetError(String),

    #[error("Desktop environment not supported: {de}")]
    UnsupportedDE { de: String },

    #[error("GSettings schema not found: {schema}")]
    SchemaNotFound { schema: String },

    #[error("Failed to detect desktop environment")]
    DetectionFailed,
}
