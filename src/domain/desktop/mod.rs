use std::path::Path;

use crate::domain::error::DEError;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::WallpaperDE;
#[cfg(target_os = "windows")]
pub use windows::WindowsDE;

pub trait DesktopEnvironment: Send + Sync {
    fn set_wallpaper(&self, image_path: &Path) -> Result<(), DEError>;
    fn is_available(&self) -> bool;
}

pub fn detect_desktop_environment() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        let raw = std::env::var("XDG_CURRENT_DESKTOP")
            .or_else(|_| std::env::var("DESKTOP_SESSION"))
            .ok()?;

        let primary = raw.split(':').next()?.trim();
        Some(primary.to_lowercase())
    }
    #[cfg(target_os = "windows")]
    {
        Some("windows".to_string())
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        None
    }
}

pub fn create_desktop_backend() -> Result<Box<dyn DesktopEnvironment>, DEError> {
    #[cfg(target_os = "linux")]
    {
        let backend = WallpaperDE;
        if backend.is_available() {
            Ok(Box::new(backend))
        } else {
            Err(DEError::UnsupportedDE {
                de: detect_desktop_environment().unwrap_or_else(|| "unknown".to_string()),
            })
        }
    }

    #[cfg(target_os = "windows")]
    {
        Ok(Box::new(WindowsDE))
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        Err(DEError::DetectionFailed)
    }
}