use std::collections::BTreeMap;

use cu::pre::*;

use crate::internal;

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
    Ok(state.installed_packages.get(package_name).cloned())
}

/// Install a package using `cargo install --git --rev`
#[cu::context("failed to install '{package}' with cargo")]
pub fn install_git_commit(package: &str, git: &str, rev: &str) -> cu::Result<()> {
    let mut state = cargo::instance()?;
    let (child, bar) = cu::which("cargo")?
        .command()
        .add(cu::args![
            "install", package, "--git", git, "--rev", rev, "--locked"
        ])
        .preset(cu::pio::cargo(format!("cargo install '{package}'")))
        .spawn()?;
    child.wait_nz()?;
    bar.done();
    state.installed_packages.clear();
    Ok(())
}

/// Install a package using `cargo install`
#[cu::context("failed to install '{package}' with cargo")]
pub fn install(package: &str) -> cu::Result<()> {
    let mut state = cargo::instance()?;
    let (child, bar) = cu::which("cargo")?
        .command()
        .add(cu::args!["install", package, "--locked"])
        .preset(cu::pio::cargo(format!("cargo install '{package}'")))
        .spawn()?;
    child.wait_nz()?;
    bar.done();
    state.installed_packages.clear();
    Ok(())
}

/// Install a package using `cargo binstall` (with fallback)
#[cu::context("failed to install '{package}' with cargo-binstall")]
pub fn binstall(package: &str) -> cu::Result<()> {
    let mut state = cargo::instance()?;
    let (child, bar) = cu::which("cargo-binstall")?
        .command()
        .add(cu::args![
            package,
            "--strategies",
            "crate-meta-data,compile",
            "--no-confirm",
            "--locked",
        ])
        .stdout(cu::pio::spinner(format!("cargo binstall '{package}'")))
        .stderr(cu::lv::E)
        .stdin_null()
        .spawn()?;
    child.wait_nz()?;
    bar.done();
    state.installed_packages.clear();
    Ok(())
}

/// Install a package using `cargo binstall --git` (with fallback)
#[cu::context("failed to install '{package}' with cargo-binstall")]
pub fn binstall_git(package: &str, git: &str) -> cu::Result<()> {
    let mut state = cargo::instance()?;
    let (child, bar) = cu::which("cargo-binstall")?
        .command()
        .add(cu::args![
            package,
            "--strategies",
            "crate-meta-data,compile",
            "--no-confirm",
            "--locked",
            "--git",
            git
        ])
        .stdout(cu::pio::spinner(format!("cargo binstall '{package}'")))
        .stderr(cu::lv::E)
        .stdin_null()
        .spawn()?;
    child.wait_nz()?;
    bar.done();
    state.installed_packages.clear();
    Ok(())
}

/// Uninstall a package using `cargo uninstall`
#[cu::context("failed to uninstall '{package}' with cargo")]
pub fn uninstall(package: &str) -> cu::Result<()> {
    let mut state = cargo::instance()?;
    cu::which("cargo")?
        .command()
        .add(cu::args!["uninstall", package])
        .stdout(cu::lv::D)
        .stderr(cu::lv::D)
        .stdin_null()
        .wait_nz()?;
    state.installed_packages.clear();
    cu::info!("uninstalled '{package}' with cargo");
    Ok(())
}
