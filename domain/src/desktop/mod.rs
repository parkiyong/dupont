use std::path::{Path, PathBuf};

use crate::error::DEError;

mod cosmic;
mod gnome;
pub use cosmic::CosmicDE;
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

/// Detect the current desktop environment from environment variables.
///
/// `XDG_CURRENT_DESKTOP` can be a colon-separated list per the freedesktop.org
/// specification (e.g. `ubuntu:GNOME`). This function uses the first entry.
pub fn detect_desktop_environment() -> Option<String> {
    let raw = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .ok()?;

    // XDG_CURRENT_DESKTOP can be colon-separated; use the first entry
    let primary = raw.split(':').next()?.trim();
    Some(primary.to_lowercase())
}

/// Create the appropriate desktop backend based on the detected environment.
///
/// Checks `XDG_CURRENT_DESKTOP` (with `DESKTOP_SESSION` fallback) and returns
/// a boxed backend implementing `DesktopEnvironment` for the detected DE.
pub fn create_desktop_backend() -> Result<Box<dyn DesktopEnvironment>, DEError> {
    let de = detect_desktop_environment().ok_or(DEError::DetectionFailed)?;
    // de is already lowercased by detect_desktop_environment()

    // Check for COSMIC explicitly (cosmic, pop-cosmic)
    // COSMIC is checked first since "pop:GNOME" could match COSMIC on Pop!_OS
    if de.contains("cosmic") {
        let backend = CosmicDE;
        if backend.is_available() {
            return Ok(Box::new(backend));
        }
        // Fallback: try GNOME if COSMIC backend unavailable
        let gnome = GnomeDE;
        if gnome.is_available() {
            return Ok(Box::new(gnome));
        }
    }

    // Check for GNOME (covers: GNOME, ubuntu:GNOME, debian, ubuntu, unity, pop:GNOME)
    if de.contains("gnome")
        || de.contains("ubuntu")
        || de.contains("debian")
        || de.contains("unity")
        || de.contains("pop")
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

    Err(DEError::UnsupportedDE { de })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// DESK-01: All detect_desktop_environment behaviors tested in sequence
    /// to avoid race conditions with shared env vars in parallel test execution.
    #[test]
    fn detect_desktop_environment_parsing_and_routing() {
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

        // 4. Returns None when neither env var is set
        std::env::remove_var("XDG_CURRENT_DESKTOP");
        std::env::remove_var("DESKTOP_SESSION");
        assert_eq!(
            detect_desktop_environment(),
            None,
            "No env vars should yield None"
        );
    }

    /// DESK-01: create_desktop_backend factory routing tested in sequence
    /// to avoid race conditions with shared env vars.
    #[test]
    fn create_desktop_backend_factory_routing() {
        // 1. "cosmic" should not produce UnsupportedDE
        std::env::set_var("XDG_CURRENT_DESKTOP", "cosmic");
        let result = create_desktop_backend();
        match &result {
            Err(DEError::UnsupportedDE { .. }) => {
                panic!("cosmic should not produce UnsupportedDE");
            }
            _ => {} // Acceptable: Ok(CosmicDE), SchemaNotFound (fallback gnome unavailable), etc.
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

        // 3. No env vars should produce DetectionFailed
        std::env::remove_var("XDG_CURRENT_DESKTOP");
        std::env::remove_var("DESKTOP_SESSION");
        let result = create_desktop_backend();
        let err = match result {
            Err(e) => e,
            Ok(_) => panic!("Expected error when no env set, got Ok(backend)"),
        };
        match err {
            DEError::DetectionFailed => {}
            other => panic!("Expected DetectionFailed, got: {:?}", other),
        }
    }
}
