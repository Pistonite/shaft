//! CMake makefile generator
use crate::pre::*;
register_binaries!("cmake");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    let v = check_homebrew!("cmake");
    check_outdated!(&v, metadata[cmake]::VERSION);
    Ok(Verified::UpToDate)
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::brew::install("cmake", false, ctx.bar_ref())?;
    Ok(())
}
pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    epkg::brew::uninstall("cmake", ctx.bar_ref())?;
    Ok(())
}
