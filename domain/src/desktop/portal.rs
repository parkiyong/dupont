use std::path::{Path, PathBuf};

use async_trait::async_trait;
use crate::desktop::DesktopEnvironment;
use crate::error::DEError;

/// GNOME Desktop Environment backend using GSettings.
///
/// This backend uses GSettings to set wallpapers via the org.gnome.desktop.background schema.
/// Works on GNOME and other desktop environments that support GSettings portals.
pub struct PortalDE;

#[async_trait]
impl DesktopEnvironment for PortalDE {
    async fn set_wallpaper(&self, image_path: &Path) -> Result<(), DEError> {
        use gio::prelude::SettingsExt;

        let path_str = image_path
            .to_str()
            .ok_or_else(|| DEError::SetError("Invalid image path".to_string()))?;

        // Verify the image file exists
        if !image_path.exists() {
            return Err(DEError::SetError(
                format!("Image file does not exist: {}", image_path.display()),
            ));
        }

        // Convert to file:// URI for GSettings
        let uri = format!("file://{}", path_str);

        let settings = gio::Settings::new("org.gnome.desktop.background");

        // Set wallpaper for light mode
        settings
            .set_string("picture-uri", &uri)
            .map_err(|e| {
                DEError::SetError(format!("Failed to set light wallpaper: {}", e))
            })?;

        // Set wallpaper for dark mode
        settings
            .set_string("picture-uri-dark", &uri)
            .map_err(|e| {
                DEError::SetError(format!("Failed to set dark wallpaper: {}", e))
            })?;

        Ok(())
    }

    fn get_current_wallpaper(&self) -> Result<Option<PathBuf>, DEError> {
        use gio::prelude::SettingsExt;

        let settings = gio::Settings::new("org.gnome.desktop.background");
        let uri = settings.string("picture-uri").to_string();

        if uri.is_empty() {
            return Ok(None);
        }

        // Convert file:// URI back to path
        if let Some(path) = uri.strip_prefix("file://") {
            return Ok(Some(PathBuf::from(path)));
        }

        Ok(None)
    }

    fn name(&self) -> &'static str {
        "GNOME Portal"
    }

    fn is_available(&self) -> bool {
        // Assume available on GNOME/portal-compatible systems
        // Actual error will occur during set_wallpaper if unavailable
        true
    }
}
