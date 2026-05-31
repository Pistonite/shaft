//! Configuration for Terminal (For Desktop Environment)

use crate::pre::*;

register_binaries!("kitty");
version_cache!(static CFG_VERSION = metadata::hyprland::kitty::CFG_VERSION);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    let version = check_pacman!("ttf-hack-nerd");
    check_outdated!(version.trim(), metadata[hack_font]::VERSION_PACMAN);
    let version = check_pacman!("kitty");
    check_outdated!(version.trim(), metadata[hyprland::kitty]::VERSION);

    check_config_version_cache!(CFG_VERSION);
    Ok(Verified::UpToDate)
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::pacman::install_many(
        &["ttf-hack-nerd", "kitty"],
        "installing terminal ui packages",
        ctx.bar_ref(),
    )
}

pub fn configure(_: &Context) -> cu::Result<()> {
    let mut home = cu::check!(std::env::home_dir(), "failed to get home dir")?;
    home.extend([".config", "kitty", "kitty.conf"]);
    cu::fs::write(home, include_bytes!("kitty.conf"))?;
    CFG_VERSION.update()?;
    Ok(())
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    epkg::pacman::uninstall("ttf-hack-nerd", ctx.bar_ref())?;
    epkg::pacman::uninstall("kitty", ctx.bar_ref())?;
    Ok(())
}
