use std::collections::BTreeMap;
use std::ffi::OsStr;
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
        .args(["-S", package_name, "--noconfirm", "--needed"])
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

#[cu::context("failed to install packages with pacman")]
pub fn install_many(package_names: &[impl AsRef<OsStr>], bar: Option<&Arc<cu::ProgressBar>>) -> cu::Result<()> {
    let reason = "installing multiple packages";
    sync_database(bar, &reason)?;
    let mut state = pacman::instance()?;
    let child = opfs::sudo("pacman", &reason)?
        .args(["-S", "--noconfirm", "--needed"])
        .args(package_names)
        .stdout(cu::lv::P)
        .stderr(cu::lv::P)
        .stdin_null()
        .spawn()?;
    child.wait_nz()?;
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

#[cu::context("failed to install '{package}' from AUR repo {repo}")]
pub fn install_aur(package: &str, repo: &str, clone_root: &Path, bar: Option<&Arc<cu::ProgressBar>>) -> cu::Result<()> {
    let reason = format!("installing from AUR: {repo}");
        let bar = cu::progress(&reason).parent(bar.cloned()).spawn();
        let clone_dir = clone_root.join(package);
        cu::fs::make_dir_absent_or_empty(&clone_dir)?;
        cu::which("git")?
            .command()
            .add(cu::args![
                "-C",
                clone_root,
                "clone",
                repo
            ])
            .stdoe(cu::lv::D)
            .stdin_null()
            .wait_nz()?;
        cu::which("makepkg")?
            .command()
            .current_dir(&clone_dir)
            .stdoe(cu::lv::D)
            .stdin_null()
            .wait_nz()?;
        // find the pkg file (.pkg.tar.zst)
        let pkg_file = cu::fs::read_dir(&clone_dir)?
            .filter_map(|entry| {
                let Ok(entry) = entry else {
                    return None;
                };
                let Ok(file_name) = entry.file_name().into_utf8() else {
                    return None;
                };
                if !file_name.ends_with(".pkg.tar.zst") {
                    return None;
                }
                if file_name.contains("debug") {
                    return None;
                }
                Some(entry.path())
            })
            .next();
        let pkg_file = cu::check!(pkg_file, "failed to find pkg file after makepkg")?;
        install_file(&pkg_file, Some(&bar))?;
        bar.done();
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
