//! Tool for installing cargo tools from binary releases

use crate::pre::*;

register_binaries!("cargo-binstall");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    let info = check_cargo!("cargo-binstall");
    check_outdated!(&info.version, metadata[cargo_binstall]::VERSION);
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    if cu::which("cargo-binstall").is_ok() {
        epkg::cargo::binstall("cargo-binstall", ctx.bar_ref())
    } else {
        epkg::cargo::install("cargo-binstall", ctx.bar_ref())
    }
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    epkg::cargo::uninstall("cargo-binstall")
}
