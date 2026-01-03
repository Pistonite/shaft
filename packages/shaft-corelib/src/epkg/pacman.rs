use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use cu::pre::*;

use crate::{internal, opfs};

internal::main_thread_singleton! {
    const pacman = Pacman::new();
}

pub(crate) struct Pacman {
    installed_packages: BTreeSet<String>,
    db_synced_time: Option<Instant>,
}

impl Pacman {
    pub const fn new() -> Self {
        Pacman {
            installed_packages: BTreeSet::new(),
            db_synced_time: None,
        }
    }
}

/// Check if a package is installed with pacman
pub fn is_installed(package_name: &str) -> cu::Result<bool> {
    let mut state = pacman::instance()?;
    if state.installed_packages.is_empty() {
        cu::debug!("pacman: querying installed packages");
        let stdout = crate::command_output!("pacman", ["-Qq"]);
        state
            .installed_packages
            .extend(stdout.lines().map(|x| x.trim().to_string()));
    }
    Ok(state.installed_packages.contains(package_name))
}

#[cu::error_ctx("failed to install '{package_name}' with pacman")]
pub fn install(package_name: &str) -> cu::Result<()> {
    cu::info!("installing '{package_name}' with pacman...");
    sync_database()?;
    let mut state = pacman::instance()?;
    opfs::sudo("pacman")?
        .add(cu::args!["-S", package_name, "--noconfirm"])
        .stdout(cu::lv::P)
        .stderr(cu::lv::E)
        .stdin_null()
        .wait_nz()?;
    state.installed_packages.clear();
    cu::info!("installed '{package_name}' with pacman");
    Ok(())
}

#[cu::error_ctx("failed to sync pacman database")]
fn sync_database() -> cu::Result<()> {
    let mut state = pacman::instance()?;
    if state
        .db_synced_time
        .is_none_or(|x| x.elapsed() > Duration::from_mins(10))
    {
        let (child, _, _) = opfs::sudo("pacman")?
            .args(["-Syy", "--noconfirm"])
            .stdoe(cu::pio::spinner("sync pacman database"))
            .stdin_null()
            .spawn()?;
        child.wait_nz()?;
        state.db_synced_time = Some(Instant::now());
    }
    Ok(())
}

#[cu::error_ctx("failed to uninstall '{package_name}' with pacman")]
pub fn uninstall(package_name: &str) -> cu::Result<()> {
    cu::info!("uninstalling '{package_name}' with pacman...");
    let mut state = pacman::instance()?;
    opfs::sudo("pacman")?
        .add(cu::args!["-R", package_name, "--noconfirm"])
        .stdout(cu::lv::P)
        .stderr(cu::lv::E)
        .stdin_null()
        .wait_nz()?;
    state.installed_packages.clear();
    cu::info!("uninstalled '{package_name}' with pacman");
    Ok(())
}
