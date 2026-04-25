use crate::application::ports::ImageCache;
use crate::domain::{CacheError, Source, SourceError, Wallpaper};
use std::path::PathBuf;

pub struct FetchWallpaperUseCase<C: ImageCache> {
    cache: C,
}

impl<C: ImageCache> FetchWallpaperUseCase<C> {
    pub fn new(cache: C) -> Self {
        Self { cache }
    }

    pub async fn execute(
        &self,
        source: &dyn Source,
    ) -> Result<(Wallpaper, PathBuf), FetchWallpaperError> {
        let wallpaper = source.fetch().await.map_err(FetchWallpaperError::Source)?;

        let cache_path = self
            .cache
            .get_or_download(&wallpaper)
            .await
            .map_err(FetchWallpaperError::Cache)?;

        Ok((wallpaper, cache_path))
    }
}

#[derive(Debug)]
pub enum FetchWallpaperError {
    Source(SourceError),
    Cache(CacheError),
}

impl std::fmt::Display for FetchWallpaperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchWallpaperError::Source(e) => write!(f, "Source error: {}", e),
            FetchWallpaperError::Cache(e) => write!(f, "Cache error: {}", e),
        }
    }
}

impl std::error::Error for FetchWallpaperError {}
