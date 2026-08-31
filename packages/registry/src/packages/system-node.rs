//! Use `node` found in PATH
use crate::pre::*;

register_binaries!("node");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_in_path!("node");
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    cu::check!(verify(ctx), "system-node requires `node` in PATH")?;
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    Ok(())
}
