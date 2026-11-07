//! Ninja the build tool

use crate::pre::*;

register_binaries!("ninja");
binary_dependencies!(_7z);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_in_shaft!("ninja");
    let v = command_output!("ninja", ["--version"]);
    check_outdated!(v.trim(), metadata[ninja]::VERSION);
    Ok(Verified::UpToDate)
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file(
        "ninja.zip",
        metadata::ninja::URL(),
        metadata::ninja::SHA(),
        ctx.bar(),
    )?;
    Ok(())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    // the archive only contains the executable, without a top-level directory
    let ninja_dir = ctx.install_dir().join("ninja");
    let ninja_zip = hmgr::paths::download("ninja.zip", metadata::ninja::URL());
    opfs::unarchive(&ninja_zip, ninja_dir, true)?;
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let ninja_dir = ctx.install_dir().join("ninja");
    let from = hmgr::paths::binary("ninja").into_utf8()?;
    let to = ninja_dir.join("ninja").into_utf8()?;
    ctx.add_item(Item::link_bin(from, to))?;
    Ok(())
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    ctx.move_install_to_old_if_exists()?;
    Ok(())
}
