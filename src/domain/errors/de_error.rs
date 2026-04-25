use thiserror::Error;

#[derive(Debug, Error)]
pub enum DEError {
    #[error("Failed to set wallpaper: {0}")]
    SetError(String),

    #[error("Desktop environment not supported: {de}")]
    UnsupportedDE { de: String },
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn all_de_error_variants_produce_readable_messages() {
        let variants: Vec<DEError> = vec![
            DEError::SetError("test detail".into()),
            DEError::UnsupportedDE { de: "sway".into() },
        ];

        for variant in variants {
            let msg = format!("{}", variant);
            assert!(
                msg.len() > 5,
                "DEError variant {:?} should produce a meaningful message, got: '{}'",
                variant,
                msg
            );
            assert!(
                !msg.contains("DEError"),
                "Display should not leak type name, got: {}",
                msg
            );
        }
    }
}