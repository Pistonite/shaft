//! CMake makefile generator
use crate::pre::*;
register_binaries!("cmake");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    let v = check_pacman!("cmake");
    check_outdated!(&v, metadata[cmake]::VERSION);
    Ok(Verified::UpToDate)
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::pacman::install("cmake", ctx.bar_ref())?;
    Ok(())
}
pub fn uninstall(_: &Context) -> cu::Result<()> {
    cu::warn!("not uninstalling cmake for your sanity");
    Ok(())
}
