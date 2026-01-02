mod platform;

pub use platform::*;
mod version;
pub use version::*;
mod download;
/// Env modification checks
pub mod env_mod;
pub mod installer;
pub use download::*;
mod main_thread;
pub use main_thread::*;

pub mod home;
pub mod resume;
pub mod shell_profile;
pub mod sysinfo;
pub mod util;

pub mod sudo;

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
