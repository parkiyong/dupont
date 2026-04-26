use crate::domain::errors::DEError;
use crate::domain::traits::DesktopEnvironment;
use std::path::Path;

#[allow(dead_code)]
pub struct WindowsDesktop;

impl DesktopEnvironment for WindowsDesktop {
    fn set_wallpaper(&self, image_path: &Path) -> Result<(), DEError> {
        let path_str = image_path
            .to_str()
            .ok_or_else(|| DEError::SetError("Invalid image path".to_string()))?;

        wallpaper::set_from_path(path_str)
            .map_err(|e| DEError::SetError(format!("Failed to set wallpaper: {}", e)))?;

        Ok(())
    }

    fn is_available(&self) -> bool {
        std::path::Path::new("C:\\Windows\\System32\\user32.dll").exists()
    }
}

#[allow(dead_code)]
pub fn create_desktop_backend() -> Result<Box<dyn DesktopEnvironment>, DEError> {
    Ok(Box::new(WindowsDesktop))
}
