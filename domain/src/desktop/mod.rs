use std::path::{Path, PathBuf};

use async_trait::async_trait;
use crate::error::DEError;

#[cfg(target_os = "linux")]
mod cosmic;
#[cfg(target_os = "linux")]
mod portal;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use cosmic::CosmicDE;
#[cfg(target_os = "linux")]
pub use portal::PortalDE;
#[cfg(target_os = "windows")]
pub use windows::WindowsDE;

/// Trait for desktop environment wallpaper setting operations
#[async_trait]
pub trait DesktopEnvironment: Send + Sync {
    /// Set the desktop wallpaper to the specified image path
    async fn set_wallpaper(&self, image_path: &Path) -> Result<(), DEError>;

    /// Get the current desktop wallpaper path (if available)
    fn get_current_wallpaper(&self) -> Result<Option<PathBuf>, DEError>;

    /// Human-readable name for this desktop environment
    fn name(&self) -> &'static str;

    /// Check if this DE is available/running on the system
    fn is_available(&self) -> bool;
}

/// Detect the current desktop environment from environment variables or platform.
pub fn detect_desktop_environment() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        let raw = std::env::var("XDG_CURRENT_DESKTOP")
            .or_else(|_| std::env::var("DESKTOP_SESSION"))
            .ok()?;

        // XDG_CURRENT_DESKTOP can be colon-separated; use the first entry
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

/// Create the appropriate desktop backend based on the detected environment.
pub fn create_desktop_backend() -> Result<Box<dyn DesktopEnvironment>, DEError> {
    #[cfg(target_os = "linux")]
    {
        let de = detect_desktop_environment().ok_or(DEError::DetectionFailed)?;

        // Check for COSMIC explicitly (cosmic, pop-cosmic)
        if de.contains("cosmic") {
            let backend = CosmicDE;
            if backend.is_available() {
                return Ok(Box::new(backend));
            }
            // Fallback: try portal if COSMIC backend unavailable
            let portal = PortalDE;
            if portal.is_available() {
                return Ok(Box::new(portal));
            }
        }

        // Check for GNOME (covers: GNOME, ubuntu:GNOME, debian, ubuntu, unity, pop:GNOME)
        if de.contains("gnome")
            || de.contains("ubuntu")
            || de.contains("debian")
            || de.contains("unity")
            || de.contains("pop")
        {
            let backend = PortalDE;
            if backend.is_available() {
                return Ok(Box::new(backend));
            } else {
                return Err(DEError::PortalUnavailable);
            }
        }

        Err(DEError::UnsupportedDE { de })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn detect_desktop_environment_parsing_and_routing_linux() {
        // 1. Colon-separated XDG_CURRENT_DESKTOP yields first entry, lowercased
        std::env::set_var("XDG_CURRENT_DESKTOP", "ubuntu:GNOME");
        std::env::remove_var("DESKTOP_SESSION");
        assert_eq!(
            detect_desktop_environment(),
            Some("ubuntu".to_string()),
            "XDG_CURRENT_DESKTOP=ubuntu:GNOME should yield 'ubuntu'"
        );

        // 2. DESKTOP_SESSION fallback when XDG_CURRENT_DESKTOP is absent
        std::env::remove_var("XDG_CURRENT_DESKTOP");
        std::env::set_var("DESKTOP_SESSION", "gnome");
        assert_eq!(
            detect_desktop_environment(),
            Some("gnome".to_string()),
            "DESKTOP_SESSION=gnome should yield 'gnome'"
        );

        // 3. Values are lowercased
        std::env::set_var("XDG_CURRENT_DESKTOP", "GNOME");
        std::env::remove_var("DESKTOP_SESSION");
        assert_eq!(
            detect_desktop_environment(),
            Some("gnome".to_string()),
            "XDG_CURRENT_DESKTOP=GNOME should be lowercased to 'gnome'"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn detect_desktop_environment_windows() {
        assert_eq!(detect_desktop_environment(), Some("windows".to_string()));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn create_desktop_backend_factory_routing_linux() {
        // 1. "cosmic" should not produce UnsupportedDE
        std::env::set_var("XDG_CURRENT_DESKTOP", "cosmic");
        let result = create_desktop_backend();
        match &result {
            Err(DEError::UnsupportedDE { .. }) => {
                panic!("cosmic should not produce UnsupportedDE");
            }
            _ => {} 
        }

        // 2. Unknown DE "i3" should produce UnsupportedDE
        std::env::set_var("XDG_CURRENT_DESKTOP", "i3");
        let result = create_desktop_backend();
        let err = match result {
            Err(e) => e,
            Ok(_) => panic!("Expected error for unknown DE 'i3', got Ok(backend)"),
        };
        match err {
            DEError::UnsupportedDE { de } => assert_eq!(de, "i3"),
            other => panic!("Expected UnsupportedDE for 'i3', got: {:?}", other),
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn create_desktop_backend_windows() {
        let result = create_desktop_backend();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name(), "Windows");
    }
}
