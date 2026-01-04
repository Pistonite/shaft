use enumset::EnumSet;

use crate::{BinId, Context, PkgId};

pub fn empty_bin_set() -> EnumSet<BinId> {
    Default::default()
}
pub fn empty_pkg_set() -> EnumSet<PkgId> {
    Default::default()
}
pub fn ok(_: &Context) -> cu::Result<()> {
    Ok(())
}
pub fn unsupported_platform<T>(_: &Context) -> cu::Result<T> {
    cu::bail!("the package is not supported on the current platform")
}
pub fn ok_none<T>(_: &Context) -> cu::Result<Option<T>> {
    Ok(None)
}
