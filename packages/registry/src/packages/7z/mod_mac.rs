//! 7-Zip
use crate::pre::*;

mod version;

register_binaries!("7z", "7zz", "7zzs");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_in_shaft!("7z");
    check_in_shaft!("7zz");
    check_in_shaft!("7zzs");
    version::check()
}
pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file("7z.txz", download_url(), metadata::_7z::SHA(), ctx.bar())?;
    Ok(())
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    ensure_terminated()?;
    ctx.move_install_to_old_if_exists()?;
    let install_dir = ctx.install_dir();
    let archive_path = hmgr::paths::download("7z.txz", download_url());
    opfs::unarchive(&archive_path, &install_dir, true)?;
    Ok(())
}
pub fn configure(ctx: &Context) -> cu::Result<()> {
    let install_dir = ctx.install_dir();
    let seven_zz = install_dir.join("7zz").into_utf8()?;
    ctx.add_item(Item::link_bin(
        hmgr::paths::binary("7z").into_utf8()?,
        seven_zz.clone(),
    ))?;
    ctx.add_item(Item::link_bin(
        hmgr::paths::binary("7zz").into_utf8()?,
        seven_zz.clone(),
    ))?;
    ctx.add_item(Item::link_bin(
        hmgr::paths::binary("7zzs").into_utf8()?,
        seven_zz,
    ))?;
    Ok(())
}
pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    ensure_terminated()?;
    ctx.move_install_to_old_if_exists()?;
    Ok(())
}

fn download_url() -> String {
    let repo = metadata::_7z::REPO;
    let version = metadata::_7z::VERSION;
    let version_no_dot = version.replace(".", "");
    format!("{repo}/releases/download/{version}/7z{version_no_dot}-mac.tar.xz")
}

fn ensure_terminated() -> cu::Result<()> {
    opfs::ensure_terminated("7z")?;
    opfs::ensure_terminated("7zz")?;
    opfs::ensure_terminated("7zzs")?;
    Ok(())
}
