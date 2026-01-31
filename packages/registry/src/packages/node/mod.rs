//! Volta with `node`, `npm`, `pnpm`, and `yarn`

use crate::pre::*;

register_binaries!("node", "volta", "pnpm", "yarn");

pub static ALIAS_VERSION: VersionCache =
    VersionCache::new("node-alias", metadata::volta::ALIAS_VERSION);

pub fn binary_dependencies() -> EnumSet<BinId> {
    enum_set! { BinId::_7z }
}

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_bin_in_path_and_shaft!("volta");
    check_bin_in_path_and_shaft!("node");
    check_bin_in_path_and_shaft!("pnpm");
    check_bin_in_path_and_shaft!("yarn");

    Ok(Verified::is_uptodate(ALIAS_VERSION.is_uptodate()?))
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file(
        volta_file_name(),
        volta_url()?,
        metadata::volta::SHA,
        ctx.bar(),
    )?;
    Ok(())
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    opfs::ensure_terminated(bin_name!("volta"))?;
    opfs::ensure_terminated(bin_name!("node"))?;
    opfs::ensure_terminated(bin_name!("pnpm"))?;
    opfs::ensure_terminated(bin_name!("yarn"))?;
    let install_dir = ctx.install_dir();
    let volta_archive = hmgr::paths::download(volta_file_name(), volta_url()?);
    #[cfg(not(windows))]
    {
        let temp_volta_dir = hmgr::paths::temp_dir("volta-tar");
        let temp_volta_tgz = temp_volta_dir.join("volta.tgz");
        let temp_volta_tar = temp_volta_dir.join("volta.tar");
        cu::fs::copy(&volta_archive, &temp_volta_tgz)?;
        opfs::un7z(&temp_volta_tgz, &temp_volta_dir, ctx.bar_ref())?;
        opfs::un7z(&temp_volta_tar, &install_dir, ctx.bar_ref())?;
    }
    #[cfg(windows)]
    {
        opfs::un7z(&volta_archive, &install_dir, ctx.bar_ref())?;
    }
    Ok(())
}
pub fn uninstall(_: &Context) -> cu::Result<()> {
    if cfg!(windows) {
        if let Ok(local) = cu::env_var("LOCALAPPDATA") {
            cu::fs::rec_remove(Path::new(&local).join("npm-cache"))?;
        }
    } else {
        if let Some(home) = std::env::home_dir() {
            cu::fs::rec_remove(home.join(".npm"))?;
        }
    }
    Ok(())
}
pub fn configure(ctx: &Context) -> cu::Result<()> {
    let volta_home = ctx.install_dir();
    let volta_home_str = volta_home.as_utf8()?;
    let volta_bin = volta_home.join(bin_name!("volta"));
    ctx.add_item(hmgr::Item::UserEnvVar(
        "VOLTA_HOME".to_string(),
        volta_home_str.to_string(),
    ))?;
    ctx.add_item(hmgr::Item::UserPath(volta_home.join("bin").into_utf8()?))?;
    ctx.add_item(hmgr::Item::LinkBin(
        bin_name!("volta").to_string(),
        volta_bin.clone().into_utf8()?,
    ))?;
    ctx.add_item(hmgr::Item::LinkBin(
        bin_name!("volta-migrate").to_string(),
        volta_home.join(bin_name!("volta-migrate")).into_utf8()?,
    ))?;
    ctx.add_item(hmgr::Item::LinkBin(
        bin_name!("node").to_string(),
        volta_home.join(bin_name!("volta-shim")).into_utf8()?,
    ))?;
    ctx.add_item(hmgr::Item::LinkBin(
        bin_name!("npm").to_string(),
        volta_home.join(bin_name!("volta-shim")).into_utf8()?,
    ))?;
    ctx.add_item(hmgr::Item::LinkBin(
        bin_name!("pnpm").to_string(),
        volta_home.join(bin_name!("volta-shim")).into_utf8()?,
    ))?;
    ctx.add_item(hmgr::Item::LinkBin(
        bin_name!("yarn").to_string(),
        volta_home.join(bin_name!("volta-shim")).into_utf8()?,
    ))?;

    let config_file = ctx.load_config_file_or_default(include_str!("config.toml"))?;
    {
        let mut package = "node".to_string();
        let version = config_file
            .get("node-version")
            .and_then(|x| x.as_str())
            .unwrap_or_default();
        if !version.is_empty() {
            cu::warn!("node version is pinned to {version}");
            package.push('@');
            package.push_str(version);
        }
        let (child, bar, _) = volta_bin
            .command()
            .args(["install", &package])
            .env("VOLTA_HOME", &volta_home)
            .stdoe(cu::pio::spinner("install node").configure_spinner(|b| b.parent(ctx.bar())))
            .stdin_null()
            .spawn()?;
        child.wait_nz()?;
        bar.done();
    }
    {
        let mut package = "pnpm".to_string();
        let version = config_file
            .get("pnpm-version")
            .and_then(|x| x.as_str())
            .unwrap_or_default();
        if !version.is_empty() {
            cu::warn!("pnpm version is pinned to {version}");
            package.push('@');
            package.push_str(version);
        }
        let (child, bar, _) = volta_bin
            .command()
            .args(["install", &package])
            .env("VOLTA_HOME", &volta_home)
            .stdoe(cu::pio::spinner("install pnpm").configure_spinner(|b| b.parent(ctx.bar())))
            .stdin_null()
            .spawn()?;
        child.wait_nz()?;
        bar.done();
    }
    {
        let mut package = "yarn".to_string();
        let version = config_file
            .get("yarn-version")
            .and_then(|x| x.as_str())
            .unwrap_or_default();
        if !version.is_empty() {
            cu::warn!("yarn version is pinned to {version}");
            package.push('@');
            package.push_str(version);
        }
        let (child, bar, _) = volta_bin
            .command()
            .args(["install", &package])
            .env("VOLTA_HOME", &volta_home)
            .stdoe(cu::pio::spinner("install yarn").configure_spinner(|b| b.parent(ctx.bar())))
            .stdin_null()
            .spawn()?;
        child.wait_nz()?;
        bar.done();
    }
    ALIAS_VERSION.update()?;
    Ok(())
}

pub fn clean(ctx: &Context) -> cu::Result<()> {
    if let Ok(pnpm) = cu::which("pnpm") {
        let (child, bar, _) = pnpm
            .command()
            .args(["store", "prune"])
            .stdoe(
                cu::pio::spinner("cleaning pnpm store")
                    .configure_spinner(|builder| builder.parent(ctx.bar())),
            )
            .stdin_null()
            .spawn()?;
        child.wait_nz()?;
        bar.done();
    }
    Ok(())
}

pub fn config_location(ctx: &Context) -> cu::Result<Option<PathBuf>> {
    Ok(Some(ctx.config_file()))
}

fn volta_file_name() -> &'static str {
    if cfg!(windows) {
        "volta.zip"
    } else {
        "volta.tgz"
    }
}

fn volta_url() -> cu::Result<String> {
    let version = metadata::volta::VERSION;
    let artifact = if cfg!(windows) {
        let arch = if_arm!("-arm64", else "");
        format!("volta-{version}-windows{arch}.zip")
    } else if cfg!(target_os = "linux") {
        format!("volta-{version}-linux.tar.gz")
    } else if cfg!(target_os = "macos") {
        cu::bail!("volta for macOS not implemented");
    } else {
        cu::bail!("unknown platform");
    };
    let repo = metadata::volta::REPO;
    Ok(format!("{repo}/releases/download/v{version}/{artifact}"))
}
