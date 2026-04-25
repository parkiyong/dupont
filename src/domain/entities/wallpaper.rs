use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallpaper {
    pub id: String,
    pub url: String,
    pub title: String,
    pub description: String,
    pub attribution: String,
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
