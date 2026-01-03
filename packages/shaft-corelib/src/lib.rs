/// External package managers
pub mod epkg;
/// Home Manager. Manages contents in the SHAFT_HOME directory
pub mod hmgr;
/// Operating/File System
pub mod opfs;

mod version;
pub use version::*;
mod download;
pub use download::*;

pub mod util;

pub(crate) mod internal;

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
