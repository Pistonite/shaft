//! Microsoft fork of Git

use crate::pre::*;

mod version;

version_cache!(static ALIAS_VERSION = metadata::git::ALIAS_VERSION);

register_binaries!("git", "scalar", "bash");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_in_path!("git");
    let version = command_output!("git", ["--version"]);
    if !version.contains("vfs") {
        cu::bail!(
            "current 'git' is not the vfs version (microsoft.git); please uninstall it or use the 'system-git' package"
        );
    }
    check_in_path!("scalar");
    check_verified!(version::verify()?);
    check_version_cache!(ALIAS_VERSION);
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    opfs::ensure_terminated("git.exe")?;
    opfs::ensure_terminated("scalar.exe")?;
    epkg::winget::install("Microsoft.Git", ctx.bar_ref())?;
    Ok(())
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    opfs::ensure_terminated("git.exe")?;
    opfs::ensure_terminated("scalar.exe")?;
    epkg::winget::uninstall("Microsoft.Git", ctx.bar_ref())?;
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let exe_path = opfs::find_in_wingit("bin/bash.exe")?;
    ctx.add_item(Item::shim_bin(
        bin_name!("bash"),
        ShimCommand::target(exe_path.into_utf8()?),
    ))?;
    ALIAS_VERSION.update()?;
    Ok(())
}
