use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Image decode error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("Cache directory not accessible")]
    NotAccessible,
}
