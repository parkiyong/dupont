use std::path::{Path, PathBuf};
use crate::error::DEError;

/// Trait for desktop environment wallpaper setting operations
pub trait DesktopEnvironment: Send + Sync {
    /// Set the desktop wallpaper to the specified image path
    fn set_wallpaper(&self, image_path: &Path) -> Result<(), DEError>;

    /// Get the current desktop wallpaper path (if available)
    fn get_current_wallpaper(&self) -> Result<Option<PathBuf>, DEError>;

    /// Human-readable name for this desktop environment
    fn name(&self) -> &'static str;

    /// Check if this DE is available/running on the system
    fn is_available(&self) -> bool;
}

/// Detect the current desktop environment from environment variables
pub fn detect_desktop_environment() -> Option<String> {
    std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .ok()
        .map(|s| s.to_lowercase())
}
