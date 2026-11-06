/// External package managers
pub mod epkg;
/// Home Manager. Manages contents in the SHAFT_HOME directory
pub mod hmgr;
/// Operating/File System
pub mod opfs;

/// JSON execution
pub mod jsexe;

/// Python standalone environment manager (needs UV)
pub mod pyse;

pub use hmgr::{ItemMgr, Version, VersionCache};

pub mod config;

pub(crate) mod internal;

mod core_requirements;
pub use core_requirements::check_requirements;

#[macro_export]
macro_rules! command_output {
    ($bin:expr) => {{
        let (child, stdout) = cu::which($bin)?.command()
            .stdout(cu::pio::string())
            .stdie_null()
            .spawn()?;
        child.wait_nz()?;
        stdout.join()??
    }};
    ($bin:expr, [$($args:expr),* $(,)?]) => {{
        let (child, stdout) = cu::which($bin)?.command()
            .add(cu::args![$($args),*])
            .stdout(cu::pio::string())
            .stdie_null()
            .spawn()?;
        child.wait_nz()?;
        stdout.join()??
    }}
}

/// Append .exe to the input on windows
#[macro_export]
macro_rules! bin_name {
    ($bin:literal) => {
        if cfg!(windows) {
            concat!($bin, ".exe")
        } else {
            $bin
        }
    };
    ($bin:expr) => {
        if cfg!(windows) {
            let mut b = { $bin }.to_string();
            b.push_str(".exe");
            b
        } else {
            { $bin }.to_string()
        }
    };
}
