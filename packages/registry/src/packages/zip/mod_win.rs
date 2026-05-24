//! Zip and Unzip commands
use crate::pre::*;

register_binaries!("zip", "unzip");
binary_dependencies!(Git);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_in_shaft!("zip");
    check_in_shaft!("unzip");
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    ctx.move_install_to_old_if_exists()?;
    // copy zip for windows from registry
    hmgr::repo::ensure_checkout()?;

    let mut zip_path = hmgr::paths::repo_registry_packages();
    zip_path.extend(["zip", "zip-win"]);
    let install_path = ctx.install_dir();
    cu::check!(
        cu::fs::rec_copy_inefficiently(&zip_path, &install_path),
        "failed to copy zip binaries from repo"
    )?;

    // note that GnuWin for unzip is outdated, so we are using the one from git
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let install_path = ctx.install_dir();
    ctx.add_item(Item::shim_bin(
        bin_name!("zip"),
        ShimCommand::target(install_path.join("zip.exe").into_utf8()?),
    ))?;

    // note that GnuWin for unzip is outdated, so we are using the one from git
    let exe_path = opfs::find_in_wingit("usr/bin/unzip.exe")?;
    ctx.add_item(Item::shim_bin(
        bin_name!("unzip"),
        ShimCommand::target(exe_path.into_utf8()?),
    ))?;

    Ok(())
}
