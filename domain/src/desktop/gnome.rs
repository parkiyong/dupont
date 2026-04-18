use std::path::{Path, PathBuf};

use crate::desktop::DesktopEnvironment;
use crate::error::DEError;
use gio::prelude::{FileExt, SettingsExt};

const GNOME_BACKGROUND_SCHEMA: &str = "org.gnome.desktop.background";
const GNOME_BACKGROUND_KEY: &str = "picture-uri";
const GNOME_BACKGROUND_DARK_KEY: &str = "picture-uri-dark";

/// GNOME desktop environment backend using gio::Settings
///
/// Uses GSettings to set wallpaper via org.gnome.desktop.background schema.
/// The Settings object is created per-call (not stored) because gio::Settings
/// is !Send + !Sync (GObject type), while DesktopEnvironment requires Send + Sync.
pub struct GnomeDE;

impl GnomeDE {
    /// Verify that the GNOME background schema is available on the system
    fn schema_exists() -> bool {
        gio::SettingsSchemaSource::default()
            .and_then(|source: gio::SettingsSchemaSource| {
                source.lookup(GNOME_BACKGROUND_SCHEMA, true)
            })
            .is_some()
    }

    /// Create a new gio::Settings for the background schema
    /// Returns DEError::SchemaNotFound if schema is missing
    fn create_settings() -> Result<gio::Settings, DEError> {
        let schema_source = gio::SettingsSchemaSource::default().ok_or(
            DEError::SchemaNotFound {
                schema: GNOME_BACKGROUND_SCHEMA.to_string(),
            },
        )?;

        let schema = schema_source
            .lookup(GNOME_BACKGROUND_SCHEMA, true)
            .ok_or(DEError::SchemaNotFound {
                schema: GNOME_BACKGROUND_SCHEMA.to_string(),
            })?;

        Ok(gio::Settings::new_full(
            &schema,
            None::<&gio::SettingsBackend>,
            None,
        ))
    }

    /// Convert a file path to a file:// URI with proper percent-encoding
    fn path_to_uri(path: &Path) -> Result<String, DEError> {
        let gfile = gio::File::for_path(path);
        Ok(gfile.uri().into())
    }

    /// Extract a file path from a file:// URI
    fn uri_to_path(uri: &str) -> Option<PathBuf> {
        uri.strip_prefix("file://").map(PathBuf::from)
    }
}

impl DesktopEnvironment for GnomeDE {
    fn set_wallpaper(&self, image_path: &Path) -> Result<(), DEError> {
        // Verify the image file exists
        if !image_path.exists() {
            return Err(DEError::SetError(format!(
                "Image file not found: {}",
                image_path.display()
            )));
        }

        // Create settings (per-call, not stored)
        let settings = Self::create_settings()?;

        // Convert path to file:// URI and set both light and dark keys
        let uri = Self::path_to_uri(image_path)?;
        settings.set_string(GNOME_BACKGROUND_KEY, &uri).map_err(|e| {
            DEError::SetError(format!(
                "Failed to set GNOME wallpaper via GSettings: {}",
                e
            ))
        })?;
        settings.set_string(GNOME_BACKGROUND_DARK_KEY, &uri).map_err(|e| {
            DEError::SetError(format!(
                "Failed to set GNOME dark wallpaper via GSettings: {}",
                e
            ))
        })?;

        Ok(())
    }

    fn get_current_wallpaper(&self) -> Result<Option<PathBuf>, DEError> {
        let settings = Self::create_settings()?;

        let uri = settings.string(GNOME_BACKGROUND_KEY);
        let uri_str = uri.as_str();

        if uri_str.is_empty() {
            return Ok(None);
        }

        let path = Self::uri_to_path(uri_str);
        match path {
            Some(p) => Ok(Some(p)),
            None => Ok(None),
        }
    }

    fn name(&self) -> &'static str {
        "GNOME"
    }

    fn is_available(&self) -> bool {
        Self::schema_exists()
    }
}
