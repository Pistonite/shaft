use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use cu::pre::*;

use crate::{internal, opfs};

internal::main_thread_singleton! {
    const pacman = Pacman::new();
}

pub(crate) struct Pacman {
    installed_packages: BTreeMap<String, String>,
    db_synced_time: Option<Instant>,
}

impl Pacman {
    pub const fn new() -> Self {
        Pacman {
            installed_packages: BTreeMap::new(),
            db_synced_time: None,
        }
    }
}

/// Check if a package is installed with pacman, returns the version if installed
pub fn installed_version(package_name: &str) -> cu::Result<Option<String>> {
    let mut state = pacman::instance()?;
    if state.installed_packages.is_empty() {
        cu::debug!("pacman: querying installed packages");
        let stdout = crate::command_output!("pacman", ["-Q"]);
        for line in stdout.lines() {
            let line = line.trim();
            if let Some((name, version)) = line.split_once(' ') {
                cu::trace!("pacman: queried installed package '{name}', version='{version}'");
                state
                    .installed_packages
                    .insert(name.to_string(), version.to_string());
            }
        }
    }
    let version = state.installed_packages.get(package_name);
    match version {
        Some(x) => {
            cu::debug!("pacman: package '{package_name}' installed, version='{x}'");
        }
        None => {
            cu::debug!("pacman: package '{package_name}' not installed");
        }
    }
    Ok(version.cloned())
}

#[cu::context("failed to install '{package_name}' with pacman")]
pub fn install(package_name: &str, bar: Option<&Arc<cu::ProgressBar>>) -> cu::Result<()> {
    let reason = format!("installing {package_name}");
    sync_database(bar, &reason)?;
    let mut state = pacman::instance()?;
    let (child, bar) = opfs::sudo("pacman", &reason)?
        .add(cu::args!["-S", package_name, "--noconfirm", "--needed"])
        .stdout(
            cu::pio::spinner(format!("pacman install '{package_name}'"))
                .configure_spinner(|builder| builder.keep(true).parent(bar.cloned())),
        )
        .stderr(cu::lv::W)
        .stdin_null()
        .spawn()?;
    child.wait_nz()?;
    bar.done();
    cu::info!("installed '{package_name}' with pacman");
    state.installed_packages.clear();
    Ok(())
}

#[cu::context("failed to install '{}' with pacman -U", path.display())]
pub fn install_file(path: &Path, bar: Option<&Arc<cu::ProgressBar>>) -> cu::Result<()> {
    let reason = format!("installing package file '{}'", path.display());
    sync_database(bar, &reason)?;
    let mut state = pacman::instance()?;
    let child = opfs::sudo("pacman", &reason)?
        .add(cu::args!["-U", path, "--noconfirm"])
        .stdout(cu::lv::D)
        .stderr(cu::lv::W)
        .stdin_null()
        .spawn()?;
    child.wait_nz()?;
    cu::info!("installed '{}' with pacman -U", path.display());
    state.installed_packages.clear();
    Ok(())
}

#[cu::context("failed to sync pacman database")]
fn sync_database(bar: Option<&Arc<cu::ProgressBar>>, reason: &str) -> cu::Result<()> {
    let mut state = pacman::instance()?;
    if state
        .db_synced_time
        .is_none_or(|x| x.elapsed() > Duration::from_mins(10))
    {
        let (child, bar, _) = opfs::sudo("pacman", reason)?
            .args(["-Syy", "--noconfirm"])
            .stdoe(
                cu::pio::spinner("sync pacman database")
                    .configure_spinner(|builder| builder.parent(bar.cloned())),
            )
            .stdin_null()
            .spawn()?;
        child.wait_nz()?;
        bar.done();
        state.db_synced_time = Some(Instant::now());
    }
    Ok(())
}

#[cu::context("failed to uninstall '{package_name}' with pacman")]
pub fn uninstall(package_name: &str, bar: Option<&Arc<cu::ProgressBar>>) -> cu::Result<()> {
    let mut state = pacman::instance()?;
    let (child, bar) = opfs::sudo("pacman", &format!("uninstall {package_name}"))?
        .add(cu::args!["-R", package_name, "--noconfirm"])
        .stdout(
            cu::pio::spinner(format!("pacman uninstall '{package_name}'"))
                .configure_spinner(|builder| builder.keep(true).parent(bar.cloned())),
        )
        .stderr(cu::lv::E)
        .stdin_null()
        .spawn()?;
    child.wait_nz()?;
    bar.done();
    state.installed_packages.clear();
    cu::info!("uninstalled '{package_name}' with pacman");
    Ok(())
}
