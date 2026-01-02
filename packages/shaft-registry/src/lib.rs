mod package;
pub use package::*;

use std::path::{Path, PathBuf};

/// Stub macro for build script to generate binaries provided by a package
macro_rules! register_binaries {
    ($($l:literal),*) => {};
}
pub(crate) use register_binaries;

macro_rules! check_bin_in_path {
    ($l:literal) => {
        if cu::which($l).is_err() {
            return Ok(Verified::NotInstalled);
        }
    };
}
pub(crate) use check_bin_in_path;

macro_rules! check_installed_with_pacman {
    ($l:literal) => {
        if !op::installer::pacman::is_installed($l)? {
            cu::bail!(concat!(
                "current '",
                $l,
                "' is not installed with pacman; please uninstall it"
            ))
        }
    };
    ($l:literal, $system:literal) => {
        if !op::installer::pacman::is_installed($l)? {
            cu::bail!(concat!(
                "current '",
                $l,
                "' is not installed with pacman; please uninstall it or use the '",
                $system,
                "' package"
            ))
        }
    };
}
pub(crate) use check_installed_with_pacman;

pub enum Verified {
    UpToDate,
    NotUpToDate,
    NotInstalled,
}

pub struct Context {
    /// The id of the package being operated on
    pub pkg: PkgId,
}
impl Context {
    pub fn package_name(&self) -> &'static str {
        self.pkg.to_str()
    }
    pub fn temp_dir(&self) -> PathBuf {
        op::home::temp_dir(self.package_name())
    }
    pub fn install_dir(&self) -> PathBuf {
        op::home::install_dir(self.package_name())
    }
    pub fn check_bin_location(&self, binary: &str, expected: &Path) -> cu::Result<()> {
        let actual = cu::which(binary)?;
        cu::ensure!(
            expected == actual,
            "expected location: '{}', actual location: '{}'",
            expected.display(),
            actual.display()
        );
        Ok(())
    }
}

#[path = "./packages.gen.rs"]
#[rustfmt::skip]
mod _gen;
pub use _gen::{BinId, PkgId};
#[path = "./package_stub.rs"]
pub(crate) mod _stub;

pub(crate) mod pre {
    pub(crate) use crate::{
        BinId, Context, Package, PkgId, Verified, check_bin_in_path, check_installed_with_pacman,
        register_binaries,
    };
}
