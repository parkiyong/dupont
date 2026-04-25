use crate::domain::entities::Wallpaper;
use crate::domain::errors::SourceError;
use async_trait::async_trait;

#[async_trait]
pub trait Source: Send + Sync {
    async fn fetch(&self) -> Result<Wallpaper, SourceError>;
}
