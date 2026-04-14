use async_trait::async_trait;
use crate::error::SourceError;
use crate::wallpaper::Wallpaper;

/// Trait for wallpaper sources (Bing, Spotlight, etc.)
#[async_trait]
pub trait Source: Send + Sync {
    /// Fetch the latest wallpaper from this source
    async fn fetch(&self) -> Result<Wallpaper, SourceError>;

    /// Unique identifier for this source
    fn id(&self) -> &'static str;

    /// Human-readable name for this source
    fn name(&self) -> &'static str;
}

/// Registry for managing multiple wallpaper sources
pub struct SourceRegistry {
    sources: Vec<Box<dyn Source>>,
}

impl SourceRegistry {
    pub fn new() -> Self {
        Self { sources: Vec::new() }
    }

    pub fn register(&mut self, source: Box<dyn Source>) {
        self.sources.push(source);
    }

    pub fn get(&self, id: &str) -> Option<&dyn Source> {
        self.sources.iter().find(|s| s.id() == id).map(|s| s.as_ref())
    }

    pub fn list(&self) -> Vec<&'static str> {
        self.sources.iter().map(|s| s.id()).collect()
    }
}

impl Default for SourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
