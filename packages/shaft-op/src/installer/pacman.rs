use std::collections::BTreeSet;

use cu::pre::*;

pub(crate) struct Pacman {
    installed_packages: BTreeSet<String>,
}

crate::main_thread! {
    const fn pacman() -> Pacman {
        Pacman {
            installed_packages: BTreeSet::new(),
        }
    }
}

/// Check if a package is installed with pacman
pub fn is_installed(package_name: &str) -> cu::Result<bool> {
    let mut state = pacman::instance()?;
    let not_loaded = state.installed_packages.is_empty();
    
    if not_loaded {
        cu::debug!("pacman: querying installed packages");
        let (child, stdout) = cu::which("pacman")?
            .command()
            .arg("-Qq")
            .stdout(cu::pio::string())
            .stdie_null()
            .spawn()?;
        child.wait_nz()?;
        let stdout = stdout.join()??;
        state.installed_packages .extend(stdout.lines().map(|x| x.trim().to_string()
        ));
    }
    Ok(state.installed_packages.contains(package_name))
}

pub fn install(package_name: &str) -> cu::Result<()> {
    cu::info!("installing '{package_name}' with pacman...");
    cu::check!(install_impl(package_name), "failed to install '{package_name}' with pacman")?;
    cu::info!("installed '{package_name}' with pacman");
    Ok(())
}
fn install_impl(package_name: &str) -> cu::Result<()> {
    let mut state = pacman::instance()?;
    cu::which("sudo")?
        .command()
        .args(["pacman", "-Syy", "--no-confirm"])
        .stdout(cu::lv::I)
        .stderr(cu::lv::E)
        .stdin_inherit()
        .wait_nz()?;
    cu::which("sudo")?
        .command()
        .add(cu::args!["pacman", "-S", package_name, "--no-confirm"])
        .stdout(cu::lv::I)
        .stderr(cu::lv::E)
        .stdin_inherit()
        .wait_nz()?;
    state.installed_packages.clear();
    Ok(())
}

pub fn uninstall(package_name: &str) -> cu::Result<()> {
    cu::info!("uninstalling '{package_name}' with pacman...");
    cu::check!(uninstall_impl(package_name), "failed to uninstall '{package_name}' with pacman")?;
    cu::info!("uninstalled '{package_name}' with pacman");
    Ok(())
}
fn uninstall_impl(package_name: &str) -> cu::Result<()> {
    let mut state = pacman::instance()?;
    cu::which("sudo")?
        .command()
        .add(cu::args!["pacman", "-R", package_name, "--no-confirm"])
        .stdout(cu::lv::I)
        .stderr(cu::lv::E)
        .stdin_inherit()
        .wait_nz()?;
    state.installed_packages.clear();
    Ok(())
}
