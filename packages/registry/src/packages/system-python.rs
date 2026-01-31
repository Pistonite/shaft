//! Use `python` found in PATH
use crate::pre::*;

register_binaries!("python");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_bin_in_path!("python");
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    cu::check!(verify(ctx), "system-python requires `python` in PATH")?;
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    Ok(())
}
