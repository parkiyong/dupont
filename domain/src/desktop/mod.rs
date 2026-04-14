use std::path::{Path, PathBuf};

use crate::error::DEError;

mod gnome;
pub use gnome::GnomeDE;

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

/// Create the appropriate desktop backend based on the detected environment.
///
/// Checks `XDG_CURRENT_DESKTOP` (with `DESKTOP_SESSION` fallback) and returns
/// a boxed backend implementing `DesktopEnvironment` for the detected DE.
pub fn create_desktop_backend() -> Result<Box<dyn DesktopEnvironment>, DEError> {
    let de = detect_desktop_environment().ok_or(DEError::DetectionFailed)?;
    let de_lower = de.to_lowercase();

    // Check for GNOME (covers: GNOME, ubuntu:GNOME, debian, ubuntu, etc.)
    if de_lower.contains("gnome")
        || de_lower.contains("ubuntu")
        || de_lower.contains("debian")
        || de_lower.contains("unity")
        || de_lower.contains("pop:gnome")
    {
        let backend = GnomeDE;
        if backend.is_available() {
            return Ok(Box::new(backend));
        } else {
            return Err(DEError::SchemaNotFound {
                schema: "org.gnome.desktop.background".to_string(),
            });
        }
    }

    // Check for COSMIC (covers: COSMIC, pop-cosmic, pop:GNOME on COSMIC)
    if de_lower.contains("cosmic") || de_lower.contains("pop") {
        // COSMIC backend will be added in Plan 02
        // For now, try GNOME compatibility as fallback
        let backend = GnomeDE;
        if backend.is_available() {
            return Ok(Box::new(backend));
        }
    }

    Err(DEError::UnsupportedDE { de })
}
