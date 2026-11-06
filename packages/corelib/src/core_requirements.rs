use crate::opfs;

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
    if !opfs::is_pacman() {
        if cu::which("dpkg").is_err() {
            cu::bail!("requirement not satisfied: dpkg not found in PATH");
        }
        if cu::which("apt").is_err() {
            cu::bail!("requirement not satisfied: apt not found in PATH");
        }
        if cu::which("apt-get").is_err() {
            cu::bail!("requirement not satisfied: apt-get not found in PATH");
        }
    }

    #[cfg(target_os = "linux")]
    check_ubuntu()?;

    Ok(())
}

/// If running on Ubuntu, check that the release is supported. Other distros are not checked
#[cfg(target_os = "linux")]
fn check_ubuntu() -> cu::Result<()> {
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
