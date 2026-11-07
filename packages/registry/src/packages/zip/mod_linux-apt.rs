//! Zip and Unzip commands
use crate::pre::*;

register_binaries!("zip", "unzip");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_apt!("zip");
    check_apt!("unzip");
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::apt::install_many(&["zip", "unzip"], "install zip and unzip", ctx.bar_ref())?;
    Ok(())
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    epkg::apt::uninstall("zip", ctx.bar_ref())?;
    epkg::apt::uninstall("unzip", ctx.bar_ref())?;
    Ok(())
}
