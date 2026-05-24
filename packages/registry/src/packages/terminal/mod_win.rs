//! Configuration for Windows Terminal

use crate::pre::*;

register_binaries!("clink-cmd");
version_cache!(static CFG_VERSION = metadata::terminal::CONFIG_VERSION);
binary_dependencies!(Cmake); // used to compile clink-cmd
config_dependencies!(Pwsh, Git);

mod windows_clink;
mod windows_font;

pub fn verify(ctx: &Context) -> cu::Result<Verified> {
    check_in_path!("wt");
    check_verified!(windows_font::verify()?);
    check_verified!(windows_clink::verify(ctx)?);
    check_config_version_cache!(CFG_VERSION);

    Ok(Verified::UpToDate)
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    windows_font::download(ctx)?;
    windows_clink::download(ctx)?;
    Ok(())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    if cu::which("wt").is_err() {
        cu::info!("installing Microsoft.WindowsTerminal with winget");
        opfs::ensure_terminated("wt.exe")?;
        opfs::ensure_terminated("WindowsTerminal.exe")?;
        epkg::winget::install("Microsoft.WindowsTerminal", ctx.bar_ref())?;
    }
    cu::check!(
        windows_font::install(&setting_json()?),
        "failed to install terminal font"
    )?;
    cu::check!(windows_clink::install(ctx), "failed to install clink")?;
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    cu::check!(windows_clink::configure(ctx), "failed to configure clink")?;
    let setting_path = setting_json()?;
    let config = cu::check!(
        json::parse::<json::Value>(&cu::fs::read_string(&setting_path)?),
        "failed to parse config for windows terminal"
    )?;
    let input = json! (
        {
            "config": config,
            "meta": {
                "pwsh_installed": ctx.is_installed(PkgId::Pwsh),
                "install_dir": hmgr::paths::install_dir("pwsh").as_utf8()?,
                "cmd_bin": cu::which("cmd.exe")?.as_utf8()?,
                "clink_cmd_bin": hmgr::paths::binary(bin_name!("clink-cmd")),
            }
        }
    );
    let config = cu::check!(
        jsexe::run(&input, include_str!("./config.js")),
        "failed to configure windows terminal"
    )?;
    cu::fs::write_json_pretty(setting_path, &config)?;
    CFG_VERSION.update()?;

    Ok(())
}

pub fn pre_uninstall(_: &Context) -> cu::Result<()> {
    cu::bail!("uninstalling windows terminal is not supported");
}
pub use pre_uninstall as uninstall;

fn setting_json() -> cu::Result<PathBuf> {
    let mut p = PathBuf::from(cu::env_var("LOCALAPPDATA")?);
    p.extend([
        "Packages",
        "Microsoft.WindowsTerminal_8wekyb3d8bbwe",
        "LocalState",
        "settings.json",
    ]);
    Ok(p)
}
