//! Zip and Unzip commands
use crate::pre::*;

register_binaries!("zip", "unzip");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    let v = check_pacman!("zip");
    check_outdated!(&v, metadata[coreutils::zip]::VERSION);
    let v = check_pacman!("unzip");
    check_outdated!(&v, metadata[coreutils::unzip]::VERSION);
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::pacman::install_many(&["zip", "unzip"], "install zip and unzip", ctx.bar_ref())?;
    Ok(())
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    epkg::pacman::uninstall("zip", ctx.bar_ref())?;
    epkg::pacman::uninstall("unzip", ctx.bar_ref())?;
    Ok(())
}
