use enumset::EnumSet;

use crate::Context;
use super::{BinId, PkgId};

pub fn empty_bin_set() -> EnumSet<BinId> {
    Default::default()
}
pub fn empty_pkg_set() -> EnumSet<PkgId> {
    Default::default()
}
pub fn ok(_: &Context) -> cu::Result<()> {
    Ok(())
}
#[allow(unused)]
pub fn unsupported_platform<T>(_: &Context) -> cu::Result<T> {
    cu::bail!("the package is not supported on the current platform")
}
pub fn ok_none<T>(_: &Context) -> cu::Result<Option<T>> {
    Ok(None)
}
impl crate::Package {
    /// Create a stub package definition, used to fill spots in the registry array
    /// for unsupported platforms
    #[allow(unused)]
    pub(crate) const fn stub(name: &'static str) -> Self {
        Self {
            enabled: false,
            name,
            binaries_fn: empty_bin_set,
            linux_flavors: enumset::enum_set! {},
            short_desc: "",
            long_desc: "",
            verify_fn: unsupported_platform,
            install_fn: unsupported_platform,
            uninstall_fn: unsupported_platform,
            binary_dependencies_fn: empty_bin_set,
            config_dependencies_fn: empty_pkg_set,
            download_fn: ok,
            configure_fn: ok,
            clean_fn: ok,
            config_location_fn: ok_none,
            backup_fn: ok,
            restore_fn: ok,
            pre_uninstall_fn: ok,
        }
    }
}
