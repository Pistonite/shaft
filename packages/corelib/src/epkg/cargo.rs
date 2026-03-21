use std::collections::BTreeMap;
use std::sync::Arc;

use cu::pre::*;

use crate::{hmgr, internal};

internal::main_thread_singleton! {
    const cargo = Cargo::new();
}

pub(crate) struct Cargo {
    installed_packages: BTreeMap<String, CargoInstalledInfo>,
}

impl Cargo {
    pub const fn new() -> Self {
        Cargo {
            installed_packages: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CargoInstalledInfo {
    /// Version string (without 'v' prefix)
    pub version: String,
    /// Optional source if not installed from default source
    pub source: Option<String>,
}

/// If a package is installed with cargo, get the info
pub fn installed_info(package_name: &str) -> cu::Result<Option<CargoInstalledInfo>> {
    let mut state = cargo::instance()?;
    if state.installed_packages.is_empty() {
        cu::debug!("cargo: querying installed packages");
        let stdout = crate::command_output!("cargo", ["install", "--list"]);
        for line in stdout.lines() {
            if line.starts_with(' ') {
                continue;
            }
            let Some(line) = line.trim().strip_suffix(':') else {
                continue;
            };
            let (line, source) = match line.find('(') {
                None => (line.trim(), None),
                Some(i) => {
                    let mut source = line[i + 1..].to_string();
                    if source.ends_with(')') {
                        source.pop();
                    }
                    (line[0..i].trim(), Some(source))
                }
            };
            let (name, version) = match line.find(' ') {
                None => (line.trim(), String::new()),
                Some(i) => {
                    let version = &line[i + 1..];
                    let version = version.strip_prefix('v').unwrap_or(version);
                    (line[0..i].trim(), version.to_string())
                }
            };
            state
                .installed_packages
                .insert(name.to_string(), CargoInstalledInfo { version, source });
        }
    }
    let info = state.installed_packages.get(package_name);
    match info {
        Some(x) => {
            cu::debug!("cargo: package '{package_name}': {x:?}");
        }
        None => {
            cu::debug!("cargo: package '{package_name}' not installed");
        }
    }
    Ok(info.cloned())
}

/// Install a package using `cargo install --git --rev`
#[cu::context("failed to install '{package}' with cargo")]
pub fn install_git_commit(
    package: &str,
    git: &str,
    rev: &str,
    bar: Option<&Arc<cu::ProgressBar>>,
) -> cu::Result<()> {
    let mut state = cargo::instance()?;
    let command = cu::which("cargo")?
        .command()
        // setting current dir in case the current directory the user is in has a
        // rust-toolchain file, which will override the rust toolchain being used
        .current_dir(hmgr::paths::home())
        .args([
            "install", package, "--git", git, "--rev", rev, "--locked"
        ]);
    let command = add_platform_build_args(command);
    let (child, bar) = command.preset(
            cu::pio::cargo(format!("cargo install '{package}'"))
                .configure_spinner(|builder| builder.keep(true).parent(bar.cloned())),
        )
        .spawn()?;
    child.wait_nz()?;
    bar.done();
    cu::info!("installed '{package}' with cargo");
    state.installed_packages.clear();
    Ok(())
}

/// Install a package using `cargo install`
#[cu::context("failed to install '{package}' with cargo")]
pub fn install(package: &str, bar: Option<&Arc<cu::ProgressBar>>) -> cu::Result<()> {
    let mut state = cargo::instance()?;
    let command = cu::which("cargo")?
        .command()
        .current_dir(hmgr::paths::home())
        .args(["install", package, "--locked"]);
    let command = add_platform_build_args(command);
    let (child, bar) = command
        .preset(
            cu::pio::cargo(format!("cargo install '{package}'"))
                .configure_spinner(|builder| builder.keep(true).parent(bar.cloned())),
        )
        .spawn()?;
    child.wait_nz()?;
    bar.done();
    cu::info!("installed '{package}' with cargo");
    state.installed_packages.clear();
    Ok(())
}

/// Install a package using `cargo binstall` (with fallback)
#[cu::context("failed to install '{package}' with cargo-binstall")]
pub fn binstall(package: &str, bar: Option<&Arc<cu::ProgressBar>>) -> cu::Result<()> {
    let mut state = cargo::instance()?;
    let (child, bar) = cu::which("cargo-binstall")?
        .command()
        .current_dir(hmgr::paths::home())
        .add(cu::args![
            package,
            "--strategies",
            "crate-meta-data",
            "--no-confirm",
            "--locked",
        ])
        .stdout(
            cu::pio::spinner(format!("cargo binstall '{package}'"))
                .configure_spinner(|builder| builder.keep(true).parent(bar.cloned())),
        )
        .stderr(cu::lv::E)
        .stdin_null()
        .spawn()?;
    child.wait_nz()?;
    bar.done();
    cu::info!("installed '{package}' with cargo-binstall");
    state.installed_packages.clear();
    Ok(())
}

/// Install a package using `cargo binstall --git` (with fallback)
#[cu::context("failed to install '{package}' with cargo-binstall")]
pub fn binstall_git(
    package: &str,
    git: &str,
    bar: Option<&Arc<cu::ProgressBar>>,
) -> cu::Result<()> {
    let mut state = cargo::instance()?;
    let (child, bar) = cu::which("cargo-binstall")?
        .command()
        .current_dir(hmgr::paths::home())
        .add(cu::args![
            package,
            "--strategies",
            "crate-meta-data",
            "--no-confirm",
            "--locked",
            "--git",
            git
        ])
        .stdout(
            cu::pio::spinner(format!("cargo binstall '{package}'"))
                .configure_spinner(|builder| builder.keep(true).parent(bar.cloned())),
        )
        .stderr(cu::lv::E)
        .stdin_null()
        .spawn()?;
    child.wait_nz()?;
    bar.done();
    cu::info!("installed '{package}' with cargo-binstall");
    state.installed_packages.clear();
    Ok(())
}

/// Uninstall a package using `cargo uninstall`
#[cu::context("failed to uninstall '{package}' with cargo")]
pub fn uninstall(package: &str) -> cu::Result<()> {
    let mut state = cargo::instance()?;
    cu::which("cargo")?
        .command()
        .current_dir(hmgr::paths::home())
        .add(cu::args!["uninstall", package])
        .stdout(cu::lv::D)
        .stderr(cu::lv::D)
        .stdin_null()
        .wait_nz()?;
    state.installed_packages.clear();
    cu::info!("uninstalled '{package}' with cargo");
    Ok(())
}

#[cfg(not(feature = "build-x64"))]
pub fn add_platform_build_args(command: cu::CommandBuilder) -> cu::CommandBuilder {
    command
}

#[cfg(all(feature = "build-x64", windows))]
pub static BUILD_X64_TARGET_TRIPLE: &str = "x86_64-pc-windows-msvc";
#[cfg(all(feature = "build-x64", target_os = "linux"))]
pub static BUILD_X64_TARGET_TRIPLE: &str = "x86_64-unknown-linux-gnu";
// note x86_64 apple is no longer a tier 1 target so custom rust build is needed
#[cfg(all(feature = "build-x64", target_os = "macos"))]
pub static BUILD_X64_TARGET_TRIPLE: &str = "x86_64-apple-darwin";

#[cfg(feature = "build-x64")]
pub fn add_build_args(command: cu::CommandBuilder) -> cu::CommandBuilder {
    command.args(["--target", BUILD_X64_TARGET_TRIPLE])
}
