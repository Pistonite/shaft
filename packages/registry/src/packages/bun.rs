//! Bun the JS Runtime
use crate::pre::*;

register_binaries!("bun", "bunx");
binary_dependencies!(_7z);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_in_shaft!("bun");
    check_in_shaft!("bunx");
    let version = command_output!("bun", ["--version"]);
    check_outdated!(version.trim(), metadata[bun]::VERSION);
    Ok(Verified::UpToDate)
}
pub fn download(ctx: &Context) -> cu::Result<()> {
    let file_name = bun_file_name()?;
    hmgr::download_file(file_name, bun_url()?, metadata::bun::SHA(), ctx.bar())?;
    Ok(())
}

fn bun_url() -> cu::Result<String> {
    let repo = metadata::bun::REPO;
    let version = metadata::bun::VERSION;
    let file_name = bun_file_name()?;
    Ok(format!(
        "{repo}/releases/download/bun-v{version}/{file_name}"
    ))
}

fn bun_file_name() -> cu::Result<&'static str> {
    if cfg!(windows) {
        Ok(if opfs::is_arm() {
            "bun-windows-aarch64.zip"
        } else {
            "bun-windows-x64.zip"
        })
    } else if cfg!(target_os = "linux") {
        Ok("bun-linux-x64.zip")
    } else if cfg!(target_os = "macos") {
        Ok("bun-darwin-x64.zip")
    } else {
        cu::bail!("nvim not supported on this platform")
    }
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    let file_name = bun_file_name()?;
    let file_stem = cu::check!(
        Path::new(file_name).file_stem(),
        "failed to get bun file stem"
    )?;
    let archive_path = hmgr::paths::download(file_name, bun_url()?);
    let temp_dir = hmgr::paths::temp_dir("bun-extract");
    ctx.move_install_to_old_if_exists()?;
    let install_dir = ctx.install_dir();
    opfs::unarchive_rename(
        archive_path,
        &temp_dir,
        temp_dir.join(file_stem),
        install_dir,
        true,
        None,
    )?;
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let install_bin = cu::path!((ctx.install_dir()) / bin_name!("bun"));
    let install_bin_str = install_bin.as_utf8()?;
    ctx.add_item(Item::link_bin(
        hmgr::paths::binary(bin_name!("bun")).into_utf8()?,
        install_bin_str,
    ))?;
    ctx.add_item(Item::link_bin(
        hmgr::paths::binary(bin_name!("bunx")).into_utf8()?,
        install_bin_str,
    ))?;
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    Ok(())
}
