use crate::domain::{CacheError, Wallpaper};
use async_trait::async_trait;

#[async_trait]
pub trait ImageCache: Send + Sync {
    async fn get_or_download(
        &self,
        wallpaper: &Wallpaper,
    ) -> Result<std::path::PathBuf, CacheError>;
}
