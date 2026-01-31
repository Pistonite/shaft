//! Use `pnpm` found in PATH
use crate::pre::*;

register_binaries!("pnpm");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_bin_in_path!("pnpm");
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    cu::check!(verify(ctx), "system-pnpm requires `pnpm` in PATH")?;
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    Ok(())
}
