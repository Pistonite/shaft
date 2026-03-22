mod windows;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;
#[cfg(any(target_os = "linux", target_os = "macos"))]
mod unix;
