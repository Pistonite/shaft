//! 7-Zip
use crate::pre::*;

mod version;

register_binaries!("7z");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_pacman!("7zip");
    version::check()
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    opfs::ensure_terminated("7z")?;
    epkg::pacman::install("7zip", ctx.bar_ref())?;
    Ok(())
}
pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    opfs::ensure_terminated("7z")?;
    epkg::pacman::uninstall("7zip", ctx.bar_ref())?;
    Ok(())
}
