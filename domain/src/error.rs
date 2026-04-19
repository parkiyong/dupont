use thiserror::Error;

/// Errors from wallpaper source fetching
#[derive(Debug, Error)]
pub enum SourceError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Failed to parse API response: {0}")]
    ParseError(String),

    #[error("No wallpaper found in API response")]
    NoWallpaperFound,

    #[error("Source unavailable: {source_name}")]
    Unavailable { source_name: String },

    #[error("Rate limited by {source_name}")]
    RateLimited { source_name: String },
}

/// Errors from cache operations
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Image decode error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("Cache directory not accessible")]
    NotAccessible,

    #[error("Cache size limit exceeded: {size}MB > {limit}MB")]
    SizeLimitExceeded { size: u64, limit: u64 },

    #[error("Cache item not found")]
    NotFound,
}

/// Errors from desktop environment operations
#[derive(Debug, Error)]
pub enum DEError {
    #[error("Failed to set wallpaper: {0}")]
    SetError(String),

    #[error("Desktop environment not supported: {de}")]
    UnsupportedDE { de: String },

    #[error("GSettings schema not found: {schema}")]
    SchemaNotFound { schema: String },

    #[error("Wallpaper portal not available")]
    PortalUnavailable,

    #[error("Failed to detect desktop environment")]
    DetectionFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// DESK-04: DEError::SetError Display contains the user-provided message.
    #[test]
    fn set_error_display_contains_message() {
        let err = DEError::SetError("image not found".to_string());
        let msg = format!("{}", err);
        assert!(
            msg.contains("Failed to set wallpaper"),
            "SetError display should start with 'Failed to set wallpaper', got: {}",
            msg
        );
        assert!(
            msg.contains("image not found"),
            "SetError display should contain the detail message, got: {}",
            msg
        );
    }

    /// DESK-04: DEError::UnsupportedDE Display contains the DE name.
    #[test]
    fn unsupported_de_display_contains_de_name() {
        let err = DEError::UnsupportedDE {
            de: "i3".to_string(),
        };
        let msg = format!("{}", err);
        assert!(
            msg.contains("not supported"),
            "UnsupportedDE display should mention 'not supported', got: {}",
            msg
        );
        assert!(
            msg.contains("i3"),
            "UnsupportedDE display should contain the DE name, got: {}",
            msg
        );
    }

    /// DESK-04: DEError::SchemaNotFound Display contains the schema name.
    #[test]
    fn schema_not_found_display_contains_schema() {
        let err = DEError::SchemaNotFound {
            schema: "org.gnome.desktop.background".to_string(),
        };
        let msg = format!("{}", err);
        assert!(
            msg.contains("schema not found"),
            "SchemaNotFound display should mention 'schema not found', got: {}",
            msg
        );
        assert!(
            msg.contains("org.gnome.desktop.background"),
            "SchemaNotFound display should contain the schema name, got: {}",
            msg
        );
    }

    /// DESK-04: DEError::DetectionFailed Display is a human-readable message.
    #[test]
    fn detection_failed_display_is_human_readable() {
        let err = DEError::DetectionFailed;
        let msg = format!("{}", err);
        assert!(
            msg.contains("Failed to detect"),
            "DetectionFailed display should mention 'Failed to detect', got: {}",
            msg
        );
        assert!(
            !msg.is_empty(),
            "DetectionFailed display should not be empty"
        );
    }

    /// DESK-04: All DEError variants produce non-empty, human-readable messages.
    #[test]
    fn all_de_error_variants_produce_readable_messages() {
        let variants: Vec<DEError> = vec![
            DEError::SetError("test detail".into()),
            DEError::UnsupportedDE { de: "sway".into() },
            DEError::SchemaNotFound {
                schema: "org.test.schema".into(),
            },
            DEError::PortalUnavailable,
            DEError::DetectionFailed,
        ];

        for variant in variants {
            let msg = format!("{}", variant);
            assert!(
                msg.len() > 5,
                "DEError variant {:?} should produce a meaningful message, got: '{}'",
                variant,
                msg
            );
            // No raw struct debug format should leak
            assert!(
                !msg.contains("DEError"),
                "Display should not leak type name, got: {}",
                msg
            );
        }
    }
}
