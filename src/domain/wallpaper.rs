use serde::{Deserialize, Serialize};

/// Represents a wallpaper fetched from an online source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallpaper {
    /// Unique identifier for the wallpaper (URL hash or API ID)
    pub id: String,

    /// Full URL to the wallpaper image
    pub url: String,

    /// Title/caption of the wallpaper
    pub title: String,

    /// Description/copyright information
    pub description: String,

    /// Attribution (copyright holder, photographer)
    pub attribution: String,

    /// Source that provided this wallpaper
    pub source: String,
}

impl Wallpaper {
    pub fn new(
        id: String,
        url: String,
        title: String,
        description: String,
        attribution: String,
        source: String,
    ) -> Self {
        Self {
            id,
            url,
            title,
            description,
            attribution,
            source,
        }
    }
}
