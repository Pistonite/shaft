//! CMake makefile generator

use crate::pre::*;

register_binaries!("cmake");
pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_in_shaft!("cmake" || "system-cctools");
    let v = command_output!("cmake", ["--version"]);
    let mut v = v.split_whitespace();
    if v.next() != Some("cmake") {
        cu::warn!("failed to parse cmake version");
        return Ok(Verified::NotUpToDate);
    }
    if v.next() != Some("version") {
        cu::warn!("failed to parse cmake version");
        return Ok(Verified::NotUpToDate);
    }
    let Some(v) = v.next() else {
        cu::warn!("failed to parse cmake version");
        return Ok(Verified::NotUpToDate);
    };
    check_outdated!(v, metadata[cmake]::VERSION);

    Ok(Verified::UpToDate)
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file("cmake.zip", cmake_url(), metadata::cmake::SHA, ctx.bar())?;
    Ok(())
}

fn cmake_url() -> String {
    let repo = metadata::cmake::REPO;
    let version = metadata::cmake::VERSION;
    let release_name = release_name();
    format!("{repo}/releases/download/v{version}/{release_name}.zip")
}

fn release_name() -> String {
    let version = metadata::cmake::VERSION;
    let arch = if_arm!("arm64", else "x86_64");
    format!("cmake-{version}-windows-{arch}")
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    ctx.move_install_to_old_if_exists()?;
    {
        let bar = cu::progress("unpacking cmake")
            .keep(true)
            .parent(ctx.bar())
            .spawn();
        let temp_extract_dir = hmgr::paths::temp_dir("cmake-extract");
        let cmake_zip = hmgr::paths::download("cmake.zip", cmake_url());
        opfs::unarchive(&cmake_zip, &temp_extract_dir, true)?;
        let cmake_dir_from = temp_extract_dir.join(release_name());
        cu::fs::rename(cmake_dir_from, ctx.install_dir())?;
        bar.done();
    }
    Ok(())
}
pub fn uninstall(_: &Context) -> cu::Result<()> {
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let cmake_dir = ctx.install_dir();
    let cmake_bin_dir = cmake_dir.join("bin");
    for file_name in ["cmake", "cmcldeps", "cpack", "ctest", "cmake-gui"] {
        let file_name = bin_name!(file_name);
        let to = cmake_bin_dir.join(&file_name).into_utf8()?;
        ctx.add_item(Item::shim_bin(file_name, ShimCommand::target(to)))?;
    }
    Ok(())
}
