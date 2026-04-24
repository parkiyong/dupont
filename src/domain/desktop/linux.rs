use std::path::Path;
use std::process::Command;

use crate::domain::error::DEError;
use crate::domain::desktop::{DesktopEnvironment, detect_desktop_environment};

pub struct WallpaperDE;

impl DesktopEnvironment for WallpaperDE {
    fn set_wallpaper(&self, image_path: &Path) -> Result<(), DEError> {
        let path_str = image_path.to_str().ok_or_else(|| {
            DEError::SetError("Invalid image path".to_string())
        })?;

        let uri = format!("file://{}", path_str);

        wallpaper::set_from_path(path_str).map_err(|e| {
            DEError::SetError(format!("Failed to set wallpaper: {}", e))
        })?;

        if let Some(de) = detect_desktop_environment() {
            match de.as_str() {
                s if s.contains("gnome") || s.contains("cinnamon") => {
                    set_desktop_dark_wallpaper(&uri)
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn is_available(&self) -> bool {
        wallpaper::get().is_ok()
    }
}

fn set_desktop_dark_wallpaper(uri: &str) {
    let _ = Command::new("gsettings")
        .args(["set", "org.cinnamon.desktop.background.picture-uri", "picture-uri-dark", uri])
        .output();
    let _ = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.background", "picture-uri-dark", uri])
        .output();
}