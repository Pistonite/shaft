//! Use `yarn` found in PATH
use crate::pre::*;

register_binaries!("yarn");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_bin_in_path!("yarn");
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    cu::check!(verify(ctx), "system-yarn requires `yarn` in PATH")?;
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    Ok(())
}
