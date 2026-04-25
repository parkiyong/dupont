use crate::application::ports::ImageCache;
use crate::domain::cache::Cache;
use crate::domain::{CacheConfig, CacheError, Wallpaper};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct CacheRepoImpl {
    cache: Arc<Mutex<Cache>>,
}

impl CacheRepoImpl {
    pub fn new(config: CacheConfig) -> Result<Self, CacheError> {
        Ok(Self {
            cache: Arc::new(Mutex::new(Cache::new(config)?)),
        })
    }
}

#[async_trait::async_trait]
impl ImageCache for CacheRepoImpl {
    async fn get_or_download(
        &self,
        wallpaper: &Wallpaper,
    ) -> Result<std::path::PathBuf, CacheError> {
        self.cache.lock().await.get_or_download(wallpaper).await
    }
}
