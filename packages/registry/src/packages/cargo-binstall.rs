//! Tool for installing cargo tools from binary releases

use crate::pre::*;

register_binaries!("cargo-binstall");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    let info = check_installed_with_cargo!("cargo-binstall");
    Ok(Verified::is_uptodate(!(Version(&info.version) < metadata::cargo_binstall::VERSION)))
}

pub fn install(_: &Context) -> cu::Result<()> {
    if cu::which("cargo-binstall").is_ok() {
        epkg::cargo::binstall("cargo-binstall")
    } else {
        epkg::cargo::install("cargo-binstall")
    }
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    epkg::cargo::uninstall("cargo-binstall")
}
