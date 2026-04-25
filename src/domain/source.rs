use crate::domain::error::SourceError;
use crate::domain::wallpaper::Wallpaper;
use async_trait::async_trait;

/// Trait for wallpaper sources (Bing, Spotlight, etc.)
#[async_trait]
pub trait Source: Send + Sync {
    /// Fetch the latest wallpaper from this source
    async fn fetch(&self) -> Result<Wallpaper, SourceError>;
}
