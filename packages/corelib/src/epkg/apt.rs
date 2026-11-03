use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use cu::pre::*;

use crate::{internal, opfs};

internal::main_thread_singleton! {
    const apt = Apt::new();
}

pub(crate) struct Apt {
    installed_packages: BTreeMap<String, String>,
    db_synced_time: Option<Instant>,
}

impl Apt {
    pub const fn new() -> Self {
        Apt {
            installed_packages: BTreeMap::new(),
            db_synced_time: None,
        }
    }
}

/// Flags for line-based output with no prompts, since the output is piped to
/// the cu printer and stdin is null
const APT_GET_FLAGS: &[&str] = &[
    "-y",
    // no progress indicators
    "-q",
    // no pty for dpkg, which would otherwise draw progress with escape sequences
    "-o",
    "Dpkg::Use-Pty=0",
    "-o",
    "Dpkg::Progress-Fancy=0",
    "-o",
    "APT::Color=0",
    // keep existing config files instead of prompting when they are modified
    "-o",
    "Dpkg::Options::=--force-confdef",
    "-o",
    "Dpkg::Options::=--force-confold",
    // same as --no-install-recommends
    "-o",
    "APT::Install-Recommends=0",
];

/// Make a sudo command for apt-get
///
/// `apt-get` is used because `apt` prints a warning about not having a stable
/// CLI when its output is not a terminal.
///
/// sudo resets the environment, so `env` is needed to set `DEBIAN_FRONTEND`,
/// which stops debconf from trying to open a dialog
fn sudo_apt_get(reason: &str) -> cu::Result<cu::Command<(), (), ()>> {
    let apt_get = cu::which("apt-get")?;
    let command = opfs::sudo("env", reason)?
        .name("apt-get")
        .add(cu::args!["DEBIAN_FRONTEND=noninteractive", apt_get])
        .args(APT_GET_FLAGS);
    Ok(command)
}

/// Check if a package is installed with apt, returns the version if installed
pub fn installed_version(package_name: &str) -> cu::Result<Option<String>> {
    let mut state = apt::instance()?;
    if state.installed_packages.is_empty() {
        cu::debug!("apt: querying installed packages");
        let stdout = crate::command_output!(
            "dpkg-query",
            ["-W", "-f=${db:Status-Abbrev}\t${Package}\t${Version}\n"]
        );
        for line in stdout.lines() {
            let mut parts = line.split('\t');
            let (Some(status), Some(name), Some(version)) =
                (parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            // dpkg also lists packages that are removed but still have config
            // files. The second character of the status is the current state
            if status.as_bytes().get(1) != Some(&b'i') {
                cu::trace!("apt: skipping package '{name}', status='{status}'");
                continue;
            }
            cu::trace!("apt: queried installed package '{name}', version='{version}'");
            state
                .installed_packages
                .insert(name.to_string(), version.to_string());
        }
    }
    let version = state.installed_packages.get(package_name);
    match version {
        Some(x) => {
            cu::debug!("apt: package '{package_name}' installed, version='{x}'");
        }
        None => {
            cu::debug!("apt: package '{package_name}' not installed");
        }
    }
    Ok(version.cloned())
}

#[cu::context("failed to install '{package_name}' with apt")]
pub fn install(package_name: &str, bar: Option<&Arc<cu::ProgressBar>>) -> cu::Result<()> {
    let reason = format!("installing {package_name}");
    sync_database(bar, &reason)?;
    let mut state = apt::instance()?;
    let (child, bar) = sudo_apt_get(&reason)?
        .args(["install", package_name])
        .stdout(
            cu::pio::spinner(format!("apt install '{package_name}'"))
                .configure_spinner(|builder| builder.keep(true).parent(bar.cloned())),
        )
        // maintainer scripts print routine messages to stderr, so it's not
        // printed as warning or error here
        .stderr(cu::lv::I)
        .stdin_null()
        .spawn()?;
    child.wait_nz()?;
    bar.done();
    cu::info!("installed '{package_name}' with apt");
    state.installed_packages.clear();
    Ok(())
}

#[cu::context("failed to install packages with apt")]
pub fn install_many(
    package_names: &[impl AsRef<OsStr>],
    reason: &str,
    bar: Option<&Arc<cu::ProgressBar>>,
) -> cu::Result<()> {
    // here we are not printing the packages but relying on provided reason,
    // because the reason is entirely controlled by the registry
    sync_database(bar, reason)?;
    let mut state = apt::instance()?;
    let child = sudo_apt_get(reason)?
        .arg("install")
        .args(package_names)
        .stdout(cu::lv::P)
        .stderr(cu::lv::P)
        .stdin_null()
        .spawn()?;
    child.wait_nz()?;
    state.installed_packages.clear();
    Ok(())
}

#[cu::context("failed to install '{}' with apt", path.display())]
pub fn install_file(path: &Path, bar: Option<&Arc<cu::ProgressBar>>) -> cu::Result<()> {
    let reason = format!("installing package file '{}'", path.display());
    sync_database(bar, &reason)?;
    let mut state = apt::instance()?;
    // apt-get treats the argument as a package name unless it contains a slash
    let path = path.normalize()?;
    let child = sudo_apt_get(&reason)?
        .add(cu::args!["install", &path])
        .stdout(cu::lv::D)
        .stderr(cu::lv::I)
        .stdin_null()
        .spawn()?;
    child.wait_nz()?;
    cu::info!("installed '{}' with apt", path.display());
    state.installed_packages.clear();
    Ok(())
}

#[cu::context("failed to sync apt database")]
fn sync_database(bar: Option<&Arc<cu::ProgressBar>>, reason: &str) -> cu::Result<()> {
    let mut state = apt::instance()?;
    if state
        .db_synced_time
        .is_none_or(|x| x.elapsed() > Duration::from_mins(10))
    {
        let (child, bar, _) = sudo_apt_get(reason)?
            .arg("update")
            .stdoe(
                cu::pio::spinner("sync apt database")
                    .configure_spinner(|builder| builder.parent(bar.cloned())),
            )
            .stdin_null()
            .spawn()?;
        if child.wait_nz().is_err() {
            cu::warn!("failed to sync apt database, continuing anyway");
        }
        bar.done();
        state.db_synced_time = Some(Instant::now());
    }
    Ok(())
}

#[cu::context("failed to uninstall '{package_name}' with apt")]
pub fn uninstall(package_name: &str, bar: Option<&Arc<cu::ProgressBar>>) -> cu::Result<()> {
    let mut state = apt::instance()?;
    let (child, bar) = sudo_apt_get(&format!("uninstall {package_name}"))?
        .args(["remove", package_name])
        .stdout(
            cu::pio::spinner(format!("apt uninstall '{package_name}'"))
                .configure_spinner(|builder| builder.keep(true).parent(bar.cloned())),
        )
        .stderr(cu::lv::I)
        .stdin_null()
        .spawn()?;
    child.wait_nz()?;
    bar.done();
    state.installed_packages.clear();
    cu::info!("uninstalled '{package_name}' with apt");
    Ok(())
}
