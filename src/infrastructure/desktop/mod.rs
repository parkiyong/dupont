pub mod linux;
pub mod windows;

#[cfg(target_os = "linux")]
pub use linux::create_desktop_backend;
#[cfg(not(target_os = "linux"))]
pub use windows::create_desktop_backend;

#[cfg(target_os = "linux")]
#[allow(dead_code, unused_imports)]
pub use linux::is_dark_mode;
#[cfg(not(target_os = "linux"))]
#[allow(dead_code)]
pub fn is_dark_mode() -> bool {
    false
}
