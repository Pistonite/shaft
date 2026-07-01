#[cfg(target_os = "macos")]
pub mod brew;
pub mod cargo;
#[cfg(target_os = "linux")]
pub mod pacman;
#[cfg(windows)]
pub mod winget;
