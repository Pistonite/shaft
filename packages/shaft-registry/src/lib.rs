use std::{path::{Path, PathBuf}, sync::Arc};

use cu::pre::*;
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
            cu::bail!(concat!("current '",$l,"' is not installed with pacman; please uninstall it"))
        }
    };
    ($l:literal, $system:literal) => {
        if !op::installer::pacman::is_installed($l)? {
            cu::bail!(concat!("current '",$l,"' is not installed with pacman; please uninstall it or use the '",$system,"' package"))
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
    pub binaries: EnumSet<BinId>,
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
    binary_dependencies_fn: fn(&Context) -> EnumSet<BinId>,
    config_dependencies_fn: fn(&Context) -> EnumSet<PkgId>,
    download_fn: fn(Arc<Context>) -> cu::BoxedFuture<cu::Result<()>>,
    build_fn: fn(&Context) -> cu::Result<()>,
    configure_fn: fn(&Context) -> cu::Result<()>,
    clean_fn: fn(&Context) -> cu::Result<()>,
}

impl Package {
    pub fn id(&self) -> PkgId {
        PkgId::from_str(self.name).unwrap()
    }
    pub const fn stub(name: &'static str) -> Self {
        Self { enabled: false, name, binaries: enumset::enum_set!{}, 
            linux_flavors: enumset::enum_set!{}, 
            short_desc: "", 
            long_desc: "", 
            verify_fn: _stub::unsupported_platform, 
            install_fn: _stub::unsupported_platform, 
            uninstall_fn: _stub::unsupported_platform, 
            binary_dependencies_fn: _stub::empty_bin_dependencies, 
            config_dependencies_fn: _stub::empty_pkg_dependencies, 
            download_fn: _stub::ok_future,
            build_fn: _stub::ok,
            configure_fn: _stub::ok,
            clean_fn: _stub::ok,
        }
    }

    /// Get the binaries the package depend on
    #[inline(always)]
    pub fn binary_dependencies(&self, ctx: &Context) -> EnumSet<BinId> {
        (self.binary_dependencies_fn)(ctx)
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
    pub async fn download(&self, ctx: Arc<Context>) -> cu::Result<()> {
        (self.download_fn)(ctx).await
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

    /// Get the packages that should be configured before this package
    #[inline(always)]
    pub fn config_dependencies(&self, ctx: &Context) -> EnumSet<PkgId> {
        (self.config_dependencies_fn)(ctx)
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
        cu::ensure!(expected== actual, "expected location: '{}', actual location: '{}'", expected.display(), actual.display());
        Ok(())
    }
}

#[path = "./packages.gen.rs"]
#[rustfmt::skip]
mod _gen;
pub use _gen::{BinId, PkgId};
#[path = "./package_stub.rs"]
pub mod _stub;

pub(crate) mod pre {
    pub(crate) use crate::{
        BinId, Context, Package, PkgId, Verified, metadata_binaries, check_bin_in_path, check_installed_with_pacman
    };
    pub(crate) use op::{VersionNumber as _};
}
