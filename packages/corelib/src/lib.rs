/// External package managers
pub mod epkg;
/// Home Manager. Manages contents in the SHAFT_HOME directory
pub mod hmgr;
/// Operating/File System
pub mod opfs;

/// JSON execution
pub mod jsexe;

pub use hmgr::{ItemMgr, Version};

pub(crate) mod internal;

pub fn check_requirements() -> cu::Result<()> {
    if opfs::which_sudo().is_err() {
        if cfg!(windows) {
            cu::hint!("sudo is required.");
            cu::hint!("please refer to the following link to enable it on Windows");
            cu::hint!("https://learn.microsoft.com/en-us/windows/advanced-settings/sudo");
        } else {
            cu::hint!("sudo is required; please install sudo with your system package manager.");
        }
        cu::bail!("requirement not satisfied: sudo not found");
    }
    cu::debug!("sudo is found");
    if let Err(e) = cu::which("cargo") {
        cu::error!("cargo not found: {e:?}");
        cu::hint!("rust toolchain is required for shaft to work.");
        cu::hint!("please refer to: https://rustup.rs");
        if cfg!(windows) {
            cu::hint!("note that MSVC build tools also need to be installed on Windows.");
        }
        cu::bail!("requirement not satisfied: cargo not found in PATH");
    }

    #[cfg(windows)]
    if let Err(e) = cu::which("winget") {
        cu::error!("winget not found: {e:?}");
        cu::hint!(
            "winget is part of Windows. Please troubleshoot with https://learn.microsoft.com/en-us/windows/msix/app-installer/install-update-app-installer"
        );
        cu::bail!("requirement not satisfied: winget not found in PATH");
    }

    #[cfg(not(windows))]
    if cu::which("bash").is_err() {
        cu::bail!("requirement not satisfied: bash not found in PATH");
    }
    Ok(())
}

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
