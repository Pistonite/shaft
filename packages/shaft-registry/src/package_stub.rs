use enumset::EnumSet;
/// Implementation of `binary_dependencies` if not provided by package
pub fn empty_bin_set() -> EnumSet<crate::BinId> {
    Default::default()
}
pub fn empty_pkg_set() -> EnumSet<crate::PkgId> {
    Default::default()
}
/// Implementation of sync function if not provided by package
pub fn ok(_: &crate::Context) -> cu::Result<()> {
    Ok(())
}

pub fn unsupported_platform<T>(_: &crate::Context) -> cu::Result<T> {
    cu::bail!("this package is not supported on the current platform/flavor");
}
