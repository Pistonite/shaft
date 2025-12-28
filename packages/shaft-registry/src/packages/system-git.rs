
//! Use `git` binary found on the system PATH. The version is NOT checked.

use cu::pre::*;

use crate::pre::*;

metadata_binaries!("git");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_bin_in_path!("git");
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    cu::check!(verify(ctx), "system-git requires 'git' to be installed on the system")?;
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    Ok(())
}
