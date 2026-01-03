use std::{path::PathBuf, time::Duration};

use corelib::opfs;
use enumset::EnumSet;

use crate::{_stub, BinId, Context, PkgId, Verified};

/// Metadata for a package
pub struct Package {
    /// If the package is enabled (supported) on the current platform
    ///
    /// For easy generation, the package metadata may be generated and stubbed on
    /// unsupported platforms
    pub(crate) enabled: bool,
    /// Name of the package in kebab case.
    ///
    /// The casing is ensured by build script for packages declared in packages/
    pub name: &'static str,

    /// Binaries provided by this package. Declared by `metadata_binaries!` macro
    pub(crate) binaries_fn: fn() -> EnumSet<BinId>,

    /// Linux package manager flavors supported by this package.
    /// By default, all flavors are supported (for example,
    /// downloading a binary)
    pub(crate) linux_flavors: EnumSet<opfs::LinuxFlavor>,

    /// Short description. The first line of the doc comment
    pub short_desc: &'static str,
    /// Long description. Everything but the first line of the doc comment
    pub long_desc: &'static str,

    // required functions
    pub(crate) verify_fn: fn(&Context) -> cu::Result<Verified>,
    pub(crate) install_fn: fn(&Context) -> cu::Result<()>,
    pub(crate) uninstall_fn: fn(&Context) -> cu::Result<()>,
    // optional functions
    pub(crate) binary_dependencies_fn: fn() -> EnumSet<BinId>,
    pub(crate) config_dependencies_fn: fn() -> EnumSet<PkgId>,
    pub(crate) download_fn: fn(&Context) -> cu::Result<()>,
    pub(crate) build_fn: fn(&Context) -> cu::Result<()>,
    pub(crate) configure_fn: fn(&Context) -> cu::Result<()>,
    pub(crate) clean_fn: fn(&Context) -> cu::Result<()>,
    pub(crate) config_location_fn: fn(&Context) -> cu::Result<Option<PathBuf>>,
    pub(crate) backup_fn: fn(&Context) -> cu::Result<()>,
    pub(crate) restore_fn: fn(&Context) -> cu::Result<()>,
    pub(crate) pre_uninstall_fn: fn(&Context) -> cu::Result<()>,
}
impl Package {
    /// Create a stub package definition, used to fill spots in the registry array
    /// for unsupported platforms
    pub(crate) const fn stub(name: &'static str) -> Self {
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
            config_location_fn: _stub::ok_none,
            backup_fn: _stub::ok,
            restore_fn: _stub::ok,
            pre_uninstall_fn: _stub::ok,
        }
    }
    /// Get the package id
    pub fn id(&self) -> PkgId {
        PkgId::from_str(self.name).unwrap()
    }

    /// Get if the package is enabled in the current platform
    pub fn enabled(&self) -> bool {
        if !self.enabled {
            return false;
        }
        #[cfg(target_os = "linux")]
        {
            if !self.linux_flavors.contains(opfs::linux_flavor()) {
                return false;
            }
        }
        true
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
    /// - When the dependency is changed (synced or removed), it will cause this package to sync as well
    ///
    /// The function must not be expensive, as it may be called multiple times when building the
    /// graph
    #[inline(always)]
    pub fn config_dependencies(&self) -> EnumSet<PkgId> {
        (self.config_dependencies_fn)()
    }

    /// Verify the package is installed and up-to-date
    #[inline(always)]
    #[cu::error_ctx("failed to verify package status for '{}'", ctx.pkg)]
    pub fn verify(&self, ctx: &Context) -> cu::Result<Verified> {
        if !self.enabled() {
            cu::bail!(
                "package '{}' does not support the current platform.",
                ctx.pkg
            );
        }
        // due to file cache, this could randomly fail
        let mut error = None;
        for _ in 0..3 {
            let e = match (self.verify_fn)(ctx) {
                Ok(x) => return Ok(x),
                Err(e) => e,
            };
            cu::debug!("failed to verify '{}': {:?}", ctx.pkg, e);
            std::thread::sleep(Duration::from_secs(1));
            error = Some(e);
        }
        Err(error.unwrap())
    }

    #[inline(always)]
    #[cu::error_ctx("failed to pre-uninstall package '{}'", ctx.pkg)]
    pub fn pre_uninstall(&self, ctx: &Context) -> cu::Result<()> {
        if !self.enabled() {
            cu::bail!(
                "package '{}' does not support the current platform.",
                ctx.pkg
            );
        }
        (self.pre_uninstall_fn)(ctx)
    }

    /// Download the package, may use cache
    #[inline(always)]
    #[cu::error_ctx("failed to download '{}'", ctx.pkg)]
    pub fn download(&self, ctx: &Context) -> cu::Result<()> {
        (self.download_fn)(ctx)
    }

    /// Build the package - The expensive part of the install.
    /// This should not have side effects besides modify the downloaded
    /// package itself. It's not executed in parallel.
    #[inline(always)]
    #[cu::error_ctx("failed to build '{}'", ctx.pkg)]
    pub fn build(&self, ctx: &Context) -> cu::Result<()> {
        (self.build_fn)(ctx)
    }

    /// Install the package - after download and build
    #[inline(always)]
    #[cu::error_ctx("failed to install '{}'", ctx.pkg)]
    pub fn install(&self, ctx: &Context) -> cu::Result<()> {
        (self.install_fn)(ctx)
    }

    /// Configure the package after installing
    #[inline(always)]
    #[cu::error_ctx("failed to configure '{}'", ctx.pkg)]
    pub fn configure(&self, ctx: &Context) -> cu::Result<()> {
        (self.configure_fn)(ctx)
    }

    /// Clean up temporary files for the package. Does not uninstall it
    #[inline(always)]
    #[cu::error_ctx("failed to clean '{}'", ctx.pkg)]
    pub fn clean(&self, ctx: &Context) -> cu::Result<()> {
        (self.clean_fn)(ctx)
    }

    /// Uninstall the package
    #[inline(always)]
    #[cu::error_ctx("failed to uninstall '{}'", ctx.pkg)]
    pub fn uninstall(&self, ctx: &Context) -> cu::Result<()> {
        (self.uninstall_fn)(ctx)
    }

    /// Get the config location. Could be a directory of file and may not exist.
    ///
    /// Return `None` when the package does not have any config associated
    #[inline(always)]
    #[cu::error_ctx("failed to get config location for '{}'", ctx.pkg)]
    pub fn config_location(&self, ctx: &Context) -> cu::Result<Option<PathBuf>> {
        (self.config_location_fn)(ctx)
    }

    /// Backup the package content to prepare for remove or update
    #[inline(always)]
    #[cu::error_ctx("failed to backup '{}'", ctx.pkg)]
    pub fn backup(&self, ctx: &Context) -> cu::Result<()> {
        (self.backup_fn)(ctx)
    }

    #[inline(always)]
    #[cu::error_ctx("failed to restore '{}'", ctx.pkg)]
    pub fn restore(&self, ctx: &Context) -> cu::Result<()> {
        (self.restore_fn)(ctx)
    }
}
