//! Operations for the SHAFT_HOME directory
//!
//! The directory is structured as:
//! - `bin/`: Binaries installed and managed by the package manager (including symlinks and wrapper
//!   scripts)
//! - `temp/<package>/`: Package-specific temporary directory to store downloads, extracted repos,
//!   etc
//! - `install/<package>/`: Installed package content
//! - `config/<package>.toml`: Package user config (not config shipped by package)

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static HOME_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Initialize the SHAFT_HOME directory path.
///
/// Will fail silently and print a warning if it's already set
pub fn init(path: PathBuf) {
    cu::debug!("initializing home path: {}", path.display());
    if HOME_PATH.set(path).is_err() {
        cu::warn!(
            "SHAFT_HOME is already initialized at '{}'",
            HOME_PATH.get().unwrap().display()
        )
    }
}

pub fn home() -> &'static Path {
    HOME_PATH
        .get()
        .expect("home not initialized; please debug with -vv")
}

pub fn shaft_binary() -> PathBuf {
    if cfg!(windows) {
        home().join("shaft.exe")
    } else {
        home().join("shaft")
    }
}

pub fn shaft_binary_old() -> PathBuf {
    if cfg!(windows) {
        home().join("shaft.old.exe")
    } else {
        home().join("shaft.old")
    }
}

pub fn env_json() -> PathBuf {
    home().join("environment.json")
}

/// Get the `init` directory
pub fn init_dir() -> PathBuf {
    home().join("init")
}

/// Get the `bin` directory
pub fn bin_dir() -> PathBuf {
    home().join("bin")
}

/// Get the `temp` directory
pub fn temp_root() -> PathBuf {
    home().join("temp")
}

/// Get the `install` directory
pub fn install_root() -> PathBuf {
    home().join("install")
}

/// Get a binary inside the `bin` directory
pub fn bin(path: impl AsRef<Path>) -> PathBuf {
    let mut x = bin_dir();
    x.push(path);
    x
}

/// Get temp dir for a package
pub fn temp_dir(package: impl AsRef<Path>) -> PathBuf {
    let mut x = temp_root();
    x.push(package);
    x
}

pub fn clean_temp_dir(package: impl AsRef<Path>) -> cu::Result<()> {
    cu::fs::rec_remove(temp_dir(package))
}

pub fn clean_temp_all() -> cu::Result<()> {
    cu::fs::rec_remove(temp_root())
}

/// Get install dir for a package
pub fn install_dir(package: impl AsRef<Path>) -> PathBuf {
    let mut x = install_root();
    x.push(package);
    x
}
