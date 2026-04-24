use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

use async_trait::async_trait;
use windows_sys::Win32_UI_Shell::SystemParametersInfoW;
use windows_sys::Win32_UI_Shell::SPI_SETDESKWALLPAPER;
use windows_sys::Win32_UI_Shell::SPIF_UPDATEINIFILE;
use windows_sys::Win32_UI_Shell::SPIF_SENDCHANGE;

use crate::desktop::DesktopEnvironment;
use crate::error::DEError;

/// Windows Desktop Environment backend.
pub struct WindowsDE;

#[async_trait]
impl DesktopEnvironment for WindowsDE {
    async fn set_wallpaper(&self, image_path: &Path) -> Result<(), DEError> {
        if !image_path.exists() {
            return Err(DEError::SetError(format!(
                "Image file not found: {}",
                image_path.display()
            )));
        }

        let path_str = image_path.to_str().ok_or_else(|| {
            DEError::SetError("Invalid image path for Windows".to_string())
        })?;

        let wide_path: Vec<u16> = OsStr::new(path_str)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let success = SystemParametersInfoW(
                SPI_SETDESKWALLPAPER,
                0,
                wide_path.as_ptr() as *mut _,
                SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
            );

            if success == 0 {
                return Err(DEError::SetError("Failed to set wallpaper on Windows".to_string()));
            }
        }

        Ok(())
    }

    fn get_current_wallpaper(&self) -> Result<Option<PathBuf>, DEError> {
        // Querying the current wallpaper on Windows is more complex (requires SPI_GETDESKWALLPAPER)
        // For now, we'll return None as it's not strictly required by the current app logic.
        Ok(None)
    }

    fn set_show_preview(&mut self, _show: bool) {
        // Not applicable on Windows
    }

    fn name(&self) -> &'static str {
        "Windows"
    }

    fn is_available(&self) -> bool {
        cfg!(target_os = "windows")
    }
}
