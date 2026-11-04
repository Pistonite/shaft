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

pub fn check_requirements() -> cu::Result<()> {
    let Ok(sudo_path) = opfs::which_sudo() else {
        if cfg!(windows) {
            cu::hint!("sudo is required.");
            cu::hint!("please refer to the following link to enable it on Windows");
            cu::hint!("https://learn.microsoft.com/en-us/windows/advanced-settings/sudo");
        } else {
            cu::hint!("sudo is required; please install sudo with your system package manager.");
        }
        cu::bail!("requirement not satisfied: sudo not found");
    };
    cu::debug!("sudo: {}", sudo_path.display());
    match cu::which("cargo") {
        Err(e) => {
            cu::error!("cargo not found: {e:?}");
            cu::hint!("rust toolchain is required for shaft to work.");
            cu::hint!("please refer to: https://rustup.rs");
            if cfg!(windows) {
                cu::hint!("note that MSVC build tools also need to be installed on Windows.");
            }
            cu::bail!("requirement not satisfied: cargo not found in PATH");
        }
        Ok(cargo_path) => {
            cu::debug!("cargo: {}", cargo_path.display());
        }
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

    #[cfg(target_os = "macos")]
    if cu::which("brew").is_err() {
        cu::bail!("requirement not satisfied: homebrew is required for macos. See brew.sh");
    }

    #[cfg(target_os = "linux")]
    check_ubuntu_version()?;

    Ok(())
}

/// If running on Ubuntu, check that the release is supported. Other distros are not checked
#[cfg(target_os = "linux")]
fn check_ubuntu_version() -> cu::Result<()> {
    let os_release = match cu::fs::read_string("/etc/os-release") {
        Ok(x) => x,
        Err(e) => {
            cu::debug!("failed to read /etc/os-release, assuming not Ubuntu: {e}");
            return Ok(());
        }
    };
    let get = |key: &str| {
        os_release.lines().find_map(|line| {
            let (k, v) = line.split_once('=')?;
            (k == key).then(|| v.trim_matches(['"', '\'']))
        })
    };
    if get("ID") != Some("ubuntu") {
        return Ok(());
    }
    let version = get("VERSION_ID").unwrap_or("unknown");
    cu::debug!("ubuntu version: {version}");
    if !version.starts_with("26.") {
        cu::bail!("requirement not satisfied: Ubuntu 26 is required, but found {version}");
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
