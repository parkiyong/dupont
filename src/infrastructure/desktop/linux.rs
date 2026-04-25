use crate::domain::errors::DEError;
use crate::domain::traits::DesktopEnvironment;
use std::path::Path;
use std::process::Command;

pub struct LinuxDesktop;

impl DesktopEnvironment for LinuxDesktop {
    fn set_wallpaper(&self, image_path: &Path) -> Result<(), DEError> {
        let path_str = image_path
            .to_str()
            .ok_or_else(|| DEError::SetError("Invalid image path".to_string()))?;

        wallpaper::set_from_path(path_str)
            .map_err(|e| DEError::SetError(format!("Failed to set wallpaper: {}", e)))?;

        if let Some(de) = detect_desktop_environment() {
            if de.contains("gnome") || de.contains("cinnamon") {
                set_desktop_dark_wallpaper(&format!("file://{}", path_str));
            }
        }

        Ok(())
    }

    fn is_available(&self) -> bool {
        wallpaper::get().is_ok()
    }
}

pub fn detect_desktop_environment() -> Option<String> {
    let raw = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .ok()?;

    let primary = raw.split(':').next()?.trim();
    Some(primary.to_lowercase())
}

pub fn create_desktop_backend() -> Result<Box<dyn DesktopEnvironment>, DEError> {
    let backend = LinuxDesktop;
    if backend.is_available() {
        Ok(Box::new(backend))
    } else {
        Err(DEError::UnsupportedDE {
            de: detect_desktop_environment().unwrap_or_else(|| "unknown".to_string()),
        })
    }
}

fn set_desktop_dark_wallpaper(uri: &str) {
    let _ = Command::new("gsettings")
        .args([
            "set",
            "org.cinnamon.desktop.background.picture-uri",
            "picture-uri-dark",
            uri,
        ])
        .output();
    let _ = Command::new("gsettings")
        .args([
            "set",
            "org.gnome.desktop.background",
            "picture-uri-dark",
            uri,
        ])
        .output();
}
