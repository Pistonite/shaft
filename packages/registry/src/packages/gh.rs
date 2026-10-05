//! GitHub CLI
use crate::pre::*;

register_binaries!("gh");
binary_dependencies!(_7z);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_in_shaft!("gh");
    let stdout = command_output!("gh", ["--version"]);
    let version_line = stdout.lines().next().unwrap_or("");
    let Some(version) = version_line
        .strip_prefix("gh version ")
        .and_then(|x| x.split_whitespace().next())
    else {
        cu::warn!("gh --version returned unexpected output: {stdout}");
        return Ok(Verified::NotUpToDate);
    };
    check_outdated!(version, metadata[gh]::VERSION);
    Ok(Verified::UpToDate)
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file(gh_file_name()?, gh_url()?, metadata::gh::SHA(), ctx.bar())?;
    Ok(())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    opfs::ensure_terminated(bin_name!("gh"))?;
    let archive_path = hmgr::paths::download(gh_file_name()?, gh_url()?);
    ctx.move_install_to_old_if_exists()?;
    let install_dir = ctx.install_dir();
    if cfg!(windows) {
        // windows zip has no top-level directory
        opfs::unarchive(archive_path, install_dir, true)?;
    } else {
        let temp_dir = hmgr::paths::temp_dir("gh-extract");
        opfs::unarchive_rename(
            archive_path,
            &temp_dir,
            temp_dir.join(gh_stem()?),
            install_dir,
            true,
            None,
        )?;
    }
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let install_bin = cu::path!((ctx.install_dir()) / "bin" / bin_name!("gh")).into_utf8()?;
    ctx.add_item(Item::link_bin(
        hmgr::paths::binary(bin_name!("gh")).into_utf8()?,
        install_bin,
    ))?;
    Ok(())
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    opfs::ensure_terminated(bin_name!("gh"))?;
    ctx.move_install_to_old_if_exists()?;
    Ok(())
}

fn gh_url() -> cu::Result<String> {
    let repo = metadata::gh::REPO;
    let version = metadata::gh::VERSION;
    let file_name = gh_file_name()?;
    Ok(format!("{repo}/releases/download/v{version}/{file_name}"))
}

fn gh_file_name() -> cu::Result<String> {
    let stem = gh_stem()?;
    if cfg!(target_os = "linux") {
        Ok(format!("{stem}.tar.gz"))
    } else {
        Ok(format!("{stem}.zip"))
    }
}

fn gh_stem() -> cu::Result<String> {
    let version = metadata::gh::VERSION;
    let platform = if cfg!(windows) {
        if opfs::is_arm() {
            "windows_arm64"
        } else {
            "windows_amd64"
        }
    } else if cfg!(target_os = "linux") {
        "linux_amd64"
    } else if cfg!(target_os = "macos") {
        "macOS_arm64"
    } else {
        cu::bail!("gh not supported on this platform")
    };
    Ok(format!("gh_{version}_{platform}"))
}
