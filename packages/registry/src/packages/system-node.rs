//! Use `node` and `npm` found in PATH
use crate::pre::*;

register_binaries!("node", "npm");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_bin_in_path!("node");
    check_bin_in_path!("npm");
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    cu::check!(verify(ctx), "system-node requires `node` and `npm` in PATH")?;
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    Ok(())
}
