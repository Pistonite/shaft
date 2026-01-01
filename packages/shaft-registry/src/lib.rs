use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use enumset::EnumSet;

macro_rules! metadata_binaries {
    ($($l:literal),*) => {};
}
pub(crate) use metadata_binaries;

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

pub struct Package {
    /// If the package is enabled (supported) on the current platform
    ///
    /// For easy generation, the package metadata may be generated and stubbed on
    /// unsupported platforms
    pub enabled: bool,
    /// Name of the package in kebab case.
    ///
    /// The casing is ensured by build script for packages declared in packages/
    pub name: &'static str,

    /// Binaries provided by this package. Declared by `metadata_binaries!` macro
    pub binaries_fn: fn() -> EnumSet<BinId>,
    /// Linux package manager flavors supported by this package.
    /// By default, all flavors are supported (for example,
    /// downloading a binary)
    pub linux_flavors: EnumSet<op::LinuxFlavor>,

    /// Short description. The first line of the doc comment
    pub short_desc: &'static str,
    /// Long description. Everything but the first line of the doc comment
    pub long_desc: &'static str,

    // required functions
    verify_fn: fn(&Context) -> cu::Result<Verified>,
    install_fn: fn(&Context) -> cu::Result<()>,
    uninstall_fn: fn(&Context) -> cu::Result<()>,

    // optional functions
    binary_dependencies_fn: fn() -> EnumSet<BinId>,
    config_dependencies_fn: fn() -> EnumSet<PkgId>,
    download_fn: fn(&Context) -> cu::Result<()>,
    build_fn: fn(&Context) -> cu::Result<()>,
    configure_fn: fn(&Context) -> cu::Result<()>,
    clean_fn: fn(&Context) -> cu::Result<()>,
}

impl Package {
    pub fn id(&self) -> PkgId {
        PkgId::from_str(self.name).unwrap()
    }
    pub const fn stub(name: &'static str) -> Self {
        Self {
            enabled: false,
            name,
            binaries_fn: _stub::empty_bin_set,
            linux_flavors: enumset::enum_set! {},
            short_desc: "",
            long_desc: "",
            verify_fn: _stub::unsupported_platform,
            install_fn: _stub::unsupported_platform,
            uninstall_fn: _stub::unsupported_platform,
            binary_dependencies_fn: _stub::empty_bin_set,
            config_dependencies_fn: _stub::empty_pkg_set,
            download_fn: _stub::ok,
            build_fn: _stub::ok,
            configure_fn: _stub::ok,
            clean_fn: _stub::ok,
        }
    }

    /// Get the binaries this package provides
    #[inline(always)]
    pub fn binaries(&self) -> EnumSet<BinId> {
        (self.binaries_fn)()
    }

    /// Get the binaries the package depend on
    ///
    /// For each binary dependency, a package that provides the binary
    /// must be synced before syncing this package
    ///
    /// The function must not be expensive, as it may be called multiple times when building the
    /// graph
    #[inline(always)]
    pub fn binary_dependencies(&self) -> EnumSet<BinId> {
        (self.binary_dependencies_fn)()
    }

    /// Get the config dependencies for this package
    ///
    /// For each config dependency:
    /// - If it is installed, it must be synced before this package when syncing this package
    /// - When the dependency is synced, it will cause this package to sync as well
    ///
    /// The function must not be expensive, as it may be called multiple times when building the
    /// graph
    #[inline(always)]
    pub fn config_dependencies(&self) -> EnumSet<PkgId> {
        (self.config_dependencies_fn)()
    }

    /// Verify the package is installed and up-to-date
    #[inline(always)]
    pub fn verify(&self, ctx: &Context) -> cu::Result<Verified> {
        #[cfg(target_os = "linux")]
        {
            if !self.linux_flavors.contains(op::linux_flavor()) {
                cu::bail!("current package manager flavor is not supported.");
            }
        }
        (self.verify_fn)(ctx)
    }

    /// Download the package. This is async and could be executed in parallel
    /// for multiple packages
    #[inline(always)]
    pub fn download(&self, ctx: &Context) -> cu::Result<()> {
        (self.download_fn)(ctx)
    }

    /// Build the package - The expensive part of the install.
    /// This should not have side effects besides modify the downloaded
    /// package itself. It's not executed in parallel.
    #[inline(always)]
    pub fn build(&self, ctx: &Context) -> cu::Result<()> {
        (self.build_fn)(ctx)
    }

    /// Install the package - after download
    #[inline(always)]
    pub fn install(&self, ctx: &Context) -> cu::Result<()> {
        (self.install_fn)(ctx)
    }

    /// Configure the package after installing
    #[inline(always)]
    pub fn configure(&self, ctx: &Context) -> cu::Result<()> {
        (self.configure_fn)(ctx)
    }

    /// Clean up temporary files for the package. Does not uninstall it
    #[inline(always)]
    pub fn clean(&self, ctx: &Context) -> cu::Result<()> {
        (self.clean_fn)(ctx)
    }

    /// Uninstall the package
    #[inline(always)]
    pub fn uninstall(&self, ctx: &Context) -> cu::Result<()> {
        (self.uninstall_fn)(ctx)
    }
}

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
        metadata_binaries,
    };
}
