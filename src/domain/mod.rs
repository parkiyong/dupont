pub mod cache;
pub mod desktop;
pub mod error;
pub mod source;
pub mod sources;
pub mod wallpaper;

// Re-export commonly used types for convenience
pub use cache::Cache;
pub use desktop::create_desktop_backend;
#[cfg(target_os = "windows")]
pub use desktop::WindowsDE;
pub use source::Source;
pub use sources::{BingSource, SpotlightSource};
pub use wallpaper::Wallpaper;
