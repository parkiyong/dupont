use std::fs;
use std::path::{Path, PathBuf};

use crate::desktop::DesktopEnvironment;
use crate::error::DEError;

const COSMIC_WALLPAPER_FILE: &str = "background.ron";

/// COSMIC desktop environment backend using direct config file manipulation.
///
/// COSMIC uses its own configuration system (not GSettings/dconf).
/// Wallpaper config is stored in a RON file in the XDG config directory.
/// This backend writes the wallpaper path directly to the config file.
pub struct CosmicDE;

impl CosmicDE {
    /// Get the COSMIC config directory path.
    fn config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|p| {
            p.join("cosmic/com.system76.CosmicSettings.Background/v1")
        })
    }

    /// Get the wallpaper config file path.
    fn wallpaper_config_path() -> Option<PathBuf> {
        Self::config_dir().map(|p| p.join(COSMIC_WALLPAPER_FILE))
    }

    /// Ensure the config directory exists, creating it if necessary.
    fn ensure_config_dir() -> Result<PathBuf, DEError> {
        let config_dir = Self::config_dir().ok_or_else(|| {
            DEError::SetError("Cannot determine COSMIC config directory".to_string())
        })?;

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).map_err(|e| {
                DEError::SetError(format!(
                    "Failed to create COSMIC config directory {}: {}",
                    config_dir.display(),
                    e
                ))
            })?;
        }

        Ok(config_dir)
    }

    /// Write wallpaper config in RON format for cosmic-bg.
    fn write_wallpaper_config(image_path: &Path) -> Result<(), DEError> {
        let config_dir = Self::ensure_config_dir()?;
        let config_path = config_dir.join(COSMIC_WALLPAPER_FILE);

        // Write a simple RON-like config that cosmic-bg can read.
        // Format: single wallpaper entry with the file path.
        let path_str = image_path.to_string_lossy();
        // Escape characters that are special in RON string literals
        let escaped = path_str.replace('\\', "\\\\").replace('"', "\\\"");
        let content = format!(
            r#"Some(Wallpaper {{
    path: Some("{}"),
    color: None,
}})"#,
            escaped
        );

        fs::write(&config_path, content).map_err(|e| {
            DEError::SetError(format!(
                "Failed to write COSMIC wallpaper config to {}: {}",
                config_path.display(),
                e
            ))
        })?;

        Ok(())
    }
}

impl DesktopEnvironment for CosmicDE {
    fn set_wallpaper(&self, image_path: &Path) -> Result<(), DEError> {
        // Verify the image file exists before writing config
        if !image_path.exists() {
            return Err(DEError::SetError(format!(
                "Image file not found: {}",
                image_path.display()
            )));
        }

        Self::write_wallpaper_config(image_path)?;

        Ok(())
    }

    fn get_current_wallpaper(&self) -> Result<Option<PathBuf>, DEError> {
        let config_path = match Self::wallpaper_config_path() {
            Some(p) if p.exists() => p,
            _ => return Ok(None),
        };

        let content = fs::read_to_string(&config_path).map_err(|e| {
            DEError::SetError(format!(
                "Failed to read COSMIC wallpaper config: {}",
                e
            ))
        })?;

        // Extract the path, handling escaped quotes (\\\" and \\\\)
        if let Some(start) = content.find("path: Some(\"") {
            let path_start = start + "path: Some(\"".len();
            let remaining = &content[path_start..];
            let mut result = String::new();
            let mut chars = remaining.chars();
            while let Some(c) = chars.next() {
                if c == '\\' {
                    if let Some(escaped) = chars.next() {
                        result.push(escaped);
                    }
                } else if c == '"' {
                    break;
                } else {
                    result.push(c);
                }
            }
            if !result.is_empty() {
                return Ok(Some(PathBuf::from(result)));
            }
        }

        Ok(None)
    }

    fn name(&self) -> &'static str {
        "COSMIC"
    }

    fn is_available(&self) -> bool {
        // Check if COSMIC config directory parent exists
        // (the specific v1 dir may not exist yet, but the cosmic dir should)
        dirs::config_dir()
            .map(|p| p.join("cosmic"))
            .map(|p| p.exists())
            .unwrap_or(false)
    }
}
