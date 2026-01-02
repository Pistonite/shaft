use std::collections::BTreeMap;

use cu::pre::*;

pub(crate) struct Cargo {
    installed_packages: BTreeMap<String, CargoInstalledInfo>,
}

#[derive(Debug, Clone)]
pub struct CargoInstalledInfo {
    /// Version string (without 'v' prefix)
    pub version: String,
    /// Optional source if not installed from default source
    pub source: Option<String>,
}
crate::main_thread! {
    const fn cargo() -> Cargo {
        Cargo {
            installed_packages: BTreeMap::new(),
        }
    }
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

#[cu::error_ctx("failed to install '{package}' with cargo")]
pub fn install_git_commit(package: &str, git: &str, rev: &str) -> cu::Result<()> {
    let mut state = cargo::instance()?;
    cu::info!("installing '{package}' with cargo...");
    cu::which("cargo")?
        .command()
        .add(cu::args![
            "install", package, "--git", git, "--rev", rev, "--locked"
        ])
        .preset(cu::pio::cargo())
        .spawn()?
        .0
        .wait_nz()?;
    state.installed_packages.clear();
    cu::info!("installed '{package}' with cargo");
    Ok(())
}

#[cu::error_ctx("failed to install '{package}' with cargo")]
pub fn install(package: &str) -> cu::Result<()> {
    let mut state = cargo::instance()?;
    cu::info!("installing '{package}' with cargo...");
    cu::which("cargo")?
        .command()
        .add(cu::args!["install", package, "--locked"])
        .preset(cu::pio::cargo())
        .spawn()?
        .0
        .wait_nz()?;
    state.installed_packages.clear();
    cu::info!("installed '{package}' with cargo");
    Ok(())
}

#[cu::error_ctx("failed to install '{package}' with cargo-binstall")]
pub fn binstall(package: &str) -> cu::Result<()> {
    let mut state = cargo::instance()?;
    cu::info!("installing '{package}' with cargo-binstall...");
    cu::which("cargo-binstall")?
        .command()
        .add(cu::args![
            package,
            "--strategies",
            "crate-meta-data",
            "--no-confirm",
            "--force"
        ])
        .stdout(cu::lv::P)
        .stderr(cu::lv::E)
        .stdin_null()
        .wait_nz()?;
    state.installed_packages.clear();
    cu::info!("installed '{package}' with cargo-binstall");
    Ok(())
}

#[cu::error_ctx("failed to install '{package}' with cargo-binstall")]
pub fn binstall_git(package: &str, git: &str) -> cu::Result<()> {
    let mut state = cargo::instance()?;
    cu::info!("installing '{package}' with cargo-binstall...");
    cu::which("cargo-binstall")?
        .command()
        .add(cu::args![
            package,
            "--strategies",
            "crate-meta-data",
            "--no-confirm",
            "--force",
            "--git",
            git
        ])
        .stdout(cu::lv::P)
        .stderr(cu::lv::E)
        .stdin_null()
        .wait_nz()?;
    state.installed_packages.clear();
    cu::info!("installed '{package}' with cargo-binstall");
    Ok(())
}

#[cu::error_ctx("failed to uninstall '{package}' with cargo")]
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
