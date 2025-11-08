use std::sync::Arc;

use enumset::EnumSet;
/// Implementation of `binary_dependencies` if not provided by package
pub fn empty_bin_dependencies(_: &crate::Context) -> EnumSet<crate::BinId>  {
    Default::default()
}
pub fn empty_pkg_dependencies(_: &crate::Context) -> EnumSet<crate::PkgId>  {
    Default::default()
}
/// Implementation of async function if not provided by package
pub fn ok_future(_: Arc<crate::Context>) -> cu::BoxedFuture<cu::Result<()>> {
    Box::pin(async { Ok(()) })
}
/// Implementation of sync function if not provided by package
pub fn ok(_: &crate::Context) -> cu::Result<()> {
    Ok(())
}
