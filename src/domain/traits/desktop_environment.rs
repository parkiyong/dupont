use crate::domain::errors::DEError;
use std::path::Path;

pub trait DesktopEnvironment: Send + Sync {
    fn set_wallpaper(&self, image_path: &Path) -> Result<(), DEError>;
    fn is_available(&self) -> bool;
}
