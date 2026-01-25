pub mod cargo;
#[cfg(windows)]
pub mod winget;
#[cfg(target_os = "linux")]
pub mod pacman;
