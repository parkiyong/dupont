pub mod linux;
pub mod windows;

#[cfg(target_os = "linux")]
pub use linux::{create_desktop_backend, is_dark_mode};
#[cfg(not(target_os = "linux"))]
pub use windows::create_desktop_backend;

#[cfg(not(target_os = "linux"))]
pub fn is_dark_mode() -> bool {
    false
}
