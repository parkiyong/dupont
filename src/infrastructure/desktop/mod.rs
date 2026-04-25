pub mod linux;
pub mod windows;

#[cfg(target_os = "linux")]
pub use linux::create_desktop_backend;
#[cfg(target_os = "windows")]
pub use windows::create_desktop_backend;
