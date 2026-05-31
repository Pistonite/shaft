//! Microsoft fork of Git

use crate::pre::*;

mod version;

version_cache!(static ALIAS_VERSION = metadata::git::ALIAS_VERSION);

register_binaries!("git", "scalar", "bash");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_verified!(verify_git_installed()?);
    check_config_version_cache!(ALIAS_VERSION);
    check_in_path!("bash");
    Ok(Verified::UpToDate)
}

// installing git is slow since winget doesn't have a normal "--needed" mode
fn verify_git_installed() -> cu::Result<Verified> {
    check_in_path!("git");
    check_in_path!("scalar");
    check_verified!(version::verify(true)?);
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    if let Ok(Verified::UpToDate) = verify_git_installed() {
        return Ok(());
    }
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
