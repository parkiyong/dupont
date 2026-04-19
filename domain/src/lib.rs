pub mod cache;
pub mod desktop;
pub mod error;
pub mod source;
pub mod sources;
pub mod wallpaper;

// Re-export commonly used types for convenience
pub use cache::{Cache, CacheConfig, CacheStats};
pub use desktop::{
    create_desktop_backend, detect_desktop_environment, CosmicDE, DesktopEnvironment, PortalDE,
};
pub use error::{CacheError, DEError, SourceError};
pub use source::{Source, SourceRegistry};
pub use sources::{BingSource, SpotlightSource};
pub use wallpaper::Wallpaper;
