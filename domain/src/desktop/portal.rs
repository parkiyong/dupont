use std::fs::File;
use std::os::fd::AsFd;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use ashpd::desktop::wallpaper::{SetOn, WallpaperRequest};
use crate::desktop::DesktopEnvironment;
use crate::error::DEError;

/// GNOME Desktop Environment backend using the Wallpaper Portal.
///
/// Uses the freedesktop.org Wallpaper Portal (org.freedesktop.portal.Wallpaper)
/// via ashpd. Works in both native and Flatpak sandboxed environments.
pub struct PortalDE {
    show_preview: bool,
}

impl Default for PortalDE {
    fn default() -> Self {
        Self { show_preview: true }
    }
}

#[async_trait]
impl DesktopEnvironment for PortalDE {
    async fn set_wallpaper(&self, image_path: &Path) -> Result<(), DEError> {
        // Verify the image file exists
        if !image_path.exists() {
            return Err(DEError::SetError(
                format!("Image file does not exist: {}", image_path.display()),
            ));
        }

        // Open the file and pass it to the portal
        let file = File::open(image_path).map_err(|e| {
            DEError::SetError(format!("Failed to open image file: {}", e))
        })?;

        WallpaperRequest::default()
            .set_on(SetOn::Background)
            .show_preview(self.show_preview)
            .build_file(&file.as_fd())
            .await
            .map_err(|e| {
                DEError::SetError(format!("Failed to set wallpaper via portal: {}", e))
            })?;

        Ok(())
    }

    fn get_current_wallpaper(&self) -> Result<Option<PathBuf>, DEError> {
        // Wallpaper Portal doesn't support querying current wallpaper
        Ok(None)
    }

    fn set_show_preview(&mut self, show: bool) {
        self.show_preview = show;
    }

    fn name(&self) -> &'static str {
        "Wallpaper Portal"
    }

    fn is_available(&self) -> bool {
        cfg!(target_os = "linux")
    }
}

