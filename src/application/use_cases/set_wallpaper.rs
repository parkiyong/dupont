use crate::domain::{DEError, DesktopEnvironment};
use std::path::Path;

pub struct SetWallpaperUseCase<DE: DesktopEnvironment> {
    backend: DE,
}

impl<DE: DesktopEnvironment> SetWallpaperUseCase<DE> {
    pub fn new(backend: DE) -> Self {
        Self { backend }
    }

    pub fn execute(&self, image_path: &Path) -> Result<(), SetWallpaperError> {
        self.backend
            .set_wallpaper(image_path)
            .map_err(SetWallpaperError::Desktop)
    }
}

#[derive(Debug)]
pub enum SetWallpaperError {
    Desktop(DEError),
}

impl std::fmt::Display for SetWallpaperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetWallpaperError::Desktop(e) => write!(f, "Desktop error: {}", e),
        }
    }
}

impl std::error::Error for SetWallpaperError {}
