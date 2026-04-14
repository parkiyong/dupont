pub mod desktop;
pub mod error;
pub mod source;
pub mod sources;
pub mod wallpaper;

// Re-export commonly used types for convenience
pub use desktop::{DesktopEnvironment, detect_desktop_environment};
pub use error::{CacheError, DEError, SourceError};
pub use source::{Source, SourceRegistry};
pub use sources::{BingSource, SpotlightSource};
pub use wallpaper::Wallpaper;
