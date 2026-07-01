//! Ninja the build tool

use crate::pre::*;

register_binaries!("ninja");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    let v = check_homebrew!("ninja");
    check_outdated!(&v, metadata[ninja]::VERSION);
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::brew::install("ninja", false, ctx.bar_ref())?;
    Ok(())
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    epkg::brew::uninstall("ninja", ctx.bar_ref())?;
    Ok(())
}
