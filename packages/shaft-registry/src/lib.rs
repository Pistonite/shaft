use std::{path::{Path, PathBuf}, sync::Arc};

use cu::pre::*;
use op::Platform;
use enumset::EnumSet;

macro_rules! metadata_binaries {
    ($($l:literal),*) => {};
}
pub(crate) use metadata_binaries;
macro_rules! metadata_platforms {
    ($($l:literal),*) => {};
}
pub(crate) use metadata_platforms;

pub struct Package {
    /// Name of the package in kebab case.
    ///
    /// The casing is ensured by build script for packages declared in packages/
    pub name: &'static str,

    /// Binaries provided by this package. Declared by `metadata_binaries!` macro
    pub binaries: EnumSet<BinId>,
    /// Platforms supported by this package. Declared by `metadata_platforms!` macro.
    /// By default, all platforms are supported
    pub platforms: EnumSet<Platform>,

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

    /// Get the binaries the package depend on
    #[inline(always)]
    pub fn binary_dependencies(&self, ctx: &Context) -> EnumSet<BinId> {
        (self.binary_dependencies_fn)(ctx)
    }

    /// Verify the package is installed and up-to-date
    #[inline(always)]
    pub fn verify(&self, ctx: &Context) -> cu::Result<Verified> {
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
    pub platform: Platform,
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

pub(crate) mod pre {
    pub(crate) use crate::{
        BinId, Context, Package, PkgId, Verified, metadata_binaries, metadata_platforms,
    };
    pub(crate) use op::{Platform, VersionNumber as _};
}
