use std::fs;
use std::path::{Path, PathBuf};

use async_trait::async_trait;

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

#[async_trait]
impl DesktopEnvironment for CosmicDE {
    async fn set_wallpaper(&self, image_path: &Path) -> Result<(), DEError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    /// Helper: create a unique temp directory under /tmp and a dummy image file.
    /// Returns (temp_dir_path, image_path). Caller is responsible for cleanup.
    fn setup_temp_image() -> (PathBuf, PathBuf) {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let tmp = std::env::temp_dir().join(format!("dupont_cosmic_test_{}", n));
        fs::create_dir_all(&tmp).expect("failed to create temp dir");
        let image_path = tmp.join("test_wallpaper.jpg");
        fs::write(&image_path, b"fake-image-data").expect("failed to write temp image");
        (tmp, image_path)
    }

    /// Helper: clean up a temp directory.
    fn cleanup_temp_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }

    /// DESK-03: set_wallpaper rejects non-existent image files with a clear error.
    #[test]
    fn set_wallpaper_rejects_missing_file() {
        let cosmic = CosmicDE;
        let bad_path = PathBuf::from("/tmp/this_file_does_not_exist_12345.jpg");
        let result = cosmic.set_wallpaper(&bad_path);
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("not found"),
            "Error message should mention 'not found', got: {}",
            err_msg
        );
    }

    /// DESK-03: name returns "COSMIC".
    #[test]
    fn cosmicde_name_is_cosmic() {
        let cosmic = CosmicDE;
        assert_eq!(cosmic.name(), "COSMIC");
    }

    /// DESK-03: get_current_wallpaper returns Ok when called (None if no config).
    #[test]
    fn get_current_wallpaper_does_not_error_when_no_config() {
        let cosmic = CosmicDE;
        let result = cosmic.get_current_wallpaper();
        assert!(result.is_ok(), "get_current_wallpaper should not error: {:?}", result);
    }

    /// DESK-03: RON config roundtrip tests combined into a single test to avoid
    /// race conditions: both tests write to the same real config file path
    /// (dirs::config_dir()/cosmic/.../background.ron), so they must run sequentially.
    #[test]
    fn roundtrip_wallpaper_config_write_and_read() {
        let cosmic = CosmicDE;
        let config_path = CosmicDE::wallpaper_config_path()
            .expect("wallpaper_config_path should resolve");

        // --- Sub-test 1: Basic roundtrip with a simple path ---
        let (tmp_dir, image_path) = setup_temp_image();
        CosmicDE::write_wallpaper_config(&image_path)
            .expect("write_wallpaper_config should succeed");
        assert!(config_path.exists(), "config file should exist after write");

        let content = fs::read_to_string(&config_path).expect("should read config file");
        assert!(
            content.contains("path: Some(\""),
            "Config should contain 'path: Some(\"', got: {}",
            content
        );

        let result = cosmic.get_current_wallpaper();
        assert!(result.is_ok(), "get_current_wallpaper should not error: {:?}", result);
        let parsed = result.unwrap();
        assert!(parsed.is_some(), "Should parse a path from the config we just wrote");
        assert_eq!(
            parsed.unwrap(),
            image_path,
            "Parsed path should match the original image path"
        );
        cleanup_temp_dir(&tmp_dir);

        // --- Sub-test 2: Roundtrip with special characters in path ---
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let tmp2 = std::env::temp_dir().join(format!("dupont_cosmic_special_{}", n));
        fs::create_dir_all(&tmp2).expect("failed to create temp dir");
        let special_path = tmp2.join("my wallpaper's image.jpg");
        fs::write(&special_path, b"fake-data").expect("failed to write temp image");

        CosmicDE::write_wallpaper_config(&special_path)
            .expect("write should succeed with special chars");

        let result = cosmic.get_current_wallpaper().expect("read should succeed");
        assert_eq!(result, Some(special_path.clone()), "Special chars path should roundtrip correctly");

        cleanup_temp_dir(&tmp2);

        // Cleanup config file
        let _ = fs::remove_file(&config_path);
    }
}
