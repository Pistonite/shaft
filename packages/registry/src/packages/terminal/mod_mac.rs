//! Configuration for Terminal

use crate::pre::*;

version_cache!(static CFG_VERSION = metadata::terminal::CONFIG_VERSION);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    let version = check_homebrew!("font-hack-nerd-font");
    check_outdated!(version.trim(), metadata[hack_font]::VERSION_HOMEBREW);

    check_config_version_cache!(CFG_VERSION);
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::brew::install("font-hack-nerd-font", true, ctx.bar_ref())?;
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let install_dir = ctx.install_dir();
    let profile_path = cu::path!(install_dir / "Catppuccin.terminal");
    cu::fs::write(&profile_path, include_bytes!("macterminal.plist"))?;
    cu::which("osascript")?
        .command()
        .args([
            "-e",
            &format!(
                "tell application \"Terminal\" to load settings set POSIX file \"{}\"",
                profile_path.as_utf8()?
            ),
        ])
        .stdoe(cu::lv::P)
        .stdin_null()
        .wait_nz()?;
    cu::which("osascript")?
        .command()
        .args(["-e", &format!("tell application \"Terminal\" to set default settings to settings set \"Catppuccin\"")])
        .stdoe(cu::lv::P)
        .stdin_null()
        .wait_nz()?;
    CFG_VERSION.update()?;
}

pub fn pre_uninstall(_: &Context) -> cu::Result<()> {
    cu::bail!("uninstalling macos terminal is not supported");
}
pub use pre_uninstall as uninstall;
