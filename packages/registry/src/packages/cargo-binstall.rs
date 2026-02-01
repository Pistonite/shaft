//! Tool for installing cargo tools from binary releases

use crate::pre::*;

register_binaries!("cargo-binstall");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    let info = check_installed_with_cargo!("cargo-binstall");
    Ok(Verified::is_uptodate(
        !(Version(&info.version).lt(metadata::cargo_binstall::VERSION)),
    ))
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
