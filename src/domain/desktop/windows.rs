use std::path::Path;
use std::process::Command;

use crate::domain::desktop::DesktopEnvironment;
use crate::domain::error::DEError;

pub struct WindowsDE;

impl DesktopEnvironment for WindowsDE {
    fn set_wallpaper(&self, image_path: &Path) -> Result<(), DEError> {
        if !image_path.exists() {
            return Err(DEError::SetError(format!(
                "Image file not found: {}",
                image_path.display()
            )));
        }

        let path_str = image_path
            .to_str()
            .ok_or_else(|| DEError::SetError("Invalid image path".to_string()))?;

        let uri = format!("file://{}", path_str);

        wallpaper::set_from_path(path_str)
            .map_err(|e| DEError::SetError(format!("Failed to set wallpaper: {}", e)))?;

        set_windows_dark_wallpaper(&uri);

        Ok(())
    }

    fn is_available(&self) -> bool {
        cfg!(target_os = "windows")
    }
}

#[cfg(target_os = "windows")]
fn set_windows_dark_wallpaper(uri: &str) {
    let path = uri.strip_prefix("file://").unwrap_or(uri);
    let _ = Command::new("powershell")
        .args([
            "-Command",
            &format!(
                "Set-ItemProperty -Path 'HKCU:\\Control Panel\\Desktop\\Wallpaper' -Name Wallpaper -Value '{}'",
                path.replace('\\', "\\\\").replace('\'', "''")
            ),
        ])
        .output();
}

#[cfg(not(target_os = "windows"))]
fn set_windows_dark_wallpaper(_uri: &str) {}
