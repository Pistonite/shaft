use std::collections::BTreeMap;
use std::sync::Arc;

use cu::pre::*;

use crate::internal;

internal::main_thread_singleton! {
    const brew = Brew::new();
}

pub(crate) struct Brew {
    installed_packages: BTreeMap<String, String>,
}

impl Brew {
    pub const fn new() -> Self {
        Brew {
            installed_packages: BTreeMap::new(),
        }
    }
}

/// Check if a package is installed with brew, returns the version if installed
pub fn installed_version(package_name: &str) -> cu::Result<Option<String>> {
    let mut state = brew::instance()?;
    if state.installed_packages.is_empty() {
        cu::debug!("brew: querying installed packages");
        let stdout = crate::command_output!("brew", ["list", "--versions"]);
        for line in stdout.lines() {
            let line = line.trim();
            if let Some((name, versions)) = line.split_once(' ') {
                // brew list --versions may show multiple versions; take the last one
                let version = versions.split_whitespace().last().unwrap_or(versions);
                cu::trace!("brew: queried installed package '{name}', version='{version}'");
                state
                    .installed_packages
                    .insert(name.to_string(), version.to_string());
            }
        }
    }
    let version = state.installed_packages.get(package_name);
    match version {
        Some(x) => {
            cu::debug!("brew: package '{package_name}' installed, version='{x}'");
        }
        None => {
            cu::debug!("brew: package '{package_name}' not installed");
        }
    }
    Ok(version.cloned())
}

#[cu::context("failed to install '{package_name}' with brew")]
pub fn install(
    package_name: &str,
    is_cask: bool,
    bar: Option<&Arc<cu::ProgressBar>>,
) -> cu::Result<()> {
    let already_installed = installed_version(package_name)?.is_some();
    let subcommand = if already_installed {
        "upgrade"
    } else {
        "install"
    };
    let mut state = brew::instance()?;
    let command = cu::which("brew")?
        .command()
        .args([subcommand, package_name, "-y"]);
    let command = if is_cask {
        command.args(["--cask"])
    } else {
        command
    };
    let (child, bar, _) = command
        .stdoe(
            cu::pio::spinner(format!("brew {subcommand} '{package_name}'"))
                .configure_spinner(|builder| builder.keep(true).parent(bar.cloned())),
        )
        .stdin_null()
        .spawn()?;
    child.wait_nz()?;
    bar.done();
    cu::info!("{subcommand}d '{package_name}' with brew");
    state.installed_packages.clear();
    Ok(())
}

#[cu::context("failed to uninstall '{package_name}' with brew")]
pub fn uninstall(package_name: &str, bar: Option<&Arc<cu::ProgressBar>>) -> cu::Result<()> {
    let mut state = brew::instance()?;
    let (child, bar, _) = cu::which("brew")?
        .command()
        .args(["uninstall", package_name])
        .stdoe(
            cu::pio::spinner(format!("brew uninstall '{package_name}'"))
                .configure_spinner(|builder| builder.keep(true).parent(bar.cloned())),
        )
        .stdin_null()
        .spawn()?;
    child.wait_nz()?;
    bar.done();
    state.installed_packages.clear();
    cu::info!("uninstalled '{package_name}' with brew");
    Ok(())
}
