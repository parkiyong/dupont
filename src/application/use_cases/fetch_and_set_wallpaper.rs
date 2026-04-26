use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::application::dto::SettingsDto;
use crate::domain::Cache;
use crate::domain::traits::{DesktopEnvironment, Source};

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum FetchAndSetError {
    FetchFailed(String),
    CacheFailed(String),
    SetWallpaperFailed(String),
}

impl std::fmt::Display for FetchAndSetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchAndSetError::FetchFailed(e) => write!(f, "Failed to fetch wallpaper: {}", e),
            FetchAndSetError::CacheFailed(e) => write!(f, "Failed to cache wallpaper: {}", e),
            FetchAndSetError::SetWallpaperFailed(e) => write!(f, "Failed to set wallpaper: {}", e),
        }
    }
}

impl std::error::Error for FetchAndSetError {}

pub struct FetchWallpaperOutput {
    pub title: String,
    pub description: String,
    pub attribution: String,
    pub cache_path: PathBuf,
}

/// Orchestrates the complete wallpaper fetch and set workflow
///
/// This use case encapsulates the application-level logic for:
/// 1. Fetching a wallpaper from a given source with specified settings
/// 2. Caching the downloaded image
/// 3. Setting it as the desktop wallpaper
pub struct FetchAndSetWallpaperUseCase;

impl FetchAndSetWallpaperUseCase {
    /// Execute the complete fetch, cache, and set wallpaper flow
    pub async fn execute(
        source: Box<dyn Source>,
        cache: Arc<Mutex<Cache>>,
        desktop: Box<dyn DesktopEnvironment>,
        _settings: &SettingsDto,
    ) -> Result<FetchWallpaperOutput, FetchAndSetError> {
        // Step 1: Fetch wallpaper from source
        let wallpaper = source
            .fetch()
            .await
            .map_err(|e| FetchAndSetError::FetchFailed(e.to_string()))?;

        // Step 2: Cache the wallpaper
        let cache_path = {
            let mut guard = cache.lock().await;
            guard
                .get_or_download(&wallpaper)
                .await
                .map_err(|e| FetchAndSetError::CacheFailed(e.to_string()))?
        };

        // Step 3: Set as desktop wallpaper
        desktop
            .set_wallpaper(&cache_path)
            .map_err(|e| FetchAndSetError::SetWallpaperFailed(e.to_string()))?;

        Ok(FetchWallpaperOutput {
            title: wallpaper.title,
            description: wallpaper.description,
            attribution: wallpaper.attribution,
            cache_path,
        })
    }
}
