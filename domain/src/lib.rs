pub mod desktop;
pub mod error;
pub mod source;
pub mod wallpaper;

// Re-export commonly used types for convenience
pub use desktop::{DesktopEnvironment, detect_desktop_environment};
pub use error::{CacheError, DEError, SourceError};
pub use source::{Source, SourceRegistry};
pub use wallpaper::Wallpaper;
