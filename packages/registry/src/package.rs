use std::path::PathBuf;

use corelib::opfs;
use cu::pre::*;
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
    #[allow(unused)]
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
    pub fn verify(&self, ctx: &Context) -> cu::Result<Verified> {
        if !self.enabled() {
            cu::bail!(
                "package '{}' does not support the current platform.",
                ctx.pkg
            );
        }
        let pkg = ctx.pkg;
        cu::check!(
            (self.verify_fn)(ctx),
            "failed to verify package status for '{pkg}'"
        )
    }

    #[inline(always)]
    #[cu::context("failed to pre-uninstall package '{}'", ctx.pkg)]
    pub fn pre_uninstall(&self, ctx: &Context) -> cu::Result<()> {
        if !self.enabled() {
            cu::bail!(
                "package '{}' does not support the current platform.",
                ctx.pkg
            );
        }
        let pkg = ctx.pkg;
        cu::check!(
            (self.pre_uninstall_fn)(ctx),
            "failed to pre-uninstall package '{pkg}'"
        )
    }

    /// Download the package, may use cache
    #[inline(always)]
    #[cu::context("failed to download '{}'", ctx.pkg)]
    pub fn download(&self, ctx: &Context) -> cu::Result<()> {
        (self.download_fn)(ctx)
    }

    /// Build the package - The expensive part of the install.
    /// This should not have side effects besides modify the downloaded
    /// package itself. It's not executed in parallel.
    #[inline(always)]
    #[cu::context("failed to build '{}'", ctx.pkg)]
    pub fn build(&self, ctx: &Context) -> cu::Result<()> {
        (self.build_fn)(ctx)
    }

    /// Install the package - after download and build
    #[inline(always)]
    #[cu::context("failed to install '{}'", ctx.pkg)]
    pub fn install(&self, ctx: &Context) -> cu::Result<()> {
        (self.install_fn)(ctx)
    }

    /// Configure the package after installing
    pub fn configure(&self, ctx: &Context) -> cu::Result<()> {
        cu::check!(
            (self.configure_fn)(ctx),
            "failed to configure '{}'",
            ctx.pkg
        )?;
        Ok(())
    }

    /// Clean up temporary files for the package. Does not uninstall it
    #[inline(always)]
    #[cu::context("failed to clean '{}'", ctx.pkg)]
    pub fn clean(&self, ctx: &Context) -> cu::Result<()> {
        (self.clean_fn)(ctx)
    }

    /// Uninstall the package
    #[inline(always)]
    #[cu::context("failed to uninstall '{}'", ctx.pkg)]
    pub fn uninstall(&self, ctx: &Context) -> cu::Result<()> {
        (self.uninstall_fn)(ctx)
    }

    /// Get the config location. Could be a directory of file and may not exist.
    ///
    /// Return `None` when the package does not have any config associated
    #[inline(always)]
    #[cu::context("failed to get config location for '{}'", ctx.pkg)]
    pub fn config_location(&self, ctx: &Context) -> cu::Result<Option<PathBuf>> {
        (self.config_location_fn)(ctx)
    }

    /// Backup the package content to prepare for remove or update
    #[inline(always)]
    #[cu::context("failed to backup '{}'", ctx.pkg)]
    pub fn backup(&self, ctx: &Context) -> cu::Result<()> {
        (self.backup_fn)(ctx)
    }

    #[inline(always)]
    pub fn backup_guard<'a, 'b>(
        &'a self,
        ctx: &'b Context,
    ) -> cu::Result<PackageRestoreGuard<'a, 'b>> {
        self.backup(ctx)?;
        Ok(PackageRestoreGuard::new(self, ctx))
    }

    #[inline(always)]
    #[cu::context("failed to restore '{}'", ctx.pkg)]
    fn restore(&self, ctx: &Context) -> cu::Result<()> {
        (self.restore_fn)(ctx)
    }
}

pub struct PackageRestoreGuard<'a, 'b> {
    package: &'a Package,
    context: &'b Context,
    needs_restore: bool,
}
impl<'a, 'b> PackageRestoreGuard<'a, 'b> {
    pub fn new(package: &'a Package, context: &'b Context) -> Self {
        Self {
            package,
            context,
            needs_restore: true,
        }
    }
    /// Clear the guard without restoring the package
    pub fn clear(&mut self) {
        self.needs_restore = false;
    }
}

impl<'a, 'b> Drop for PackageRestoreGuard<'a, 'b> {
    fn drop(&mut self) {
        if self.needs_restore {
            if let Err(e) = self.package.restore(&self.context) {
                cu::error!("failed to restore package '{}': {:?}", self.context.pkg, e);
            }
        }
    }
}
