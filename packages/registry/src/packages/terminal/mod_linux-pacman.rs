//! Configuration for Terminal (For Desktop Environment)

use crate::pre::*;

binary_dependencies!(Vihypr);
version_cache!(static CFG_VERSION = metadata::hyprland::kitty::CFG_VERSION);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_version_cache!(CFG_VERSION);
    Ok(Verified::UpToDate)
}
pub fn install(_: &Context) -> cu::Result<()> {
    Ok(())
}

pub fn configure(_: &Context) -> cu::Result<()> {
    let mut home = cu::check!(std::env::home_dir(), "failed to get home dir")?;
    home.extend([".config", "kitty", "kitty.conf"]);
    cu::fs::write(home, include_bytes!("kitty.conf"))?;
    CFG_VERSION.update()?;
    Ok(())
}
pub fn uninstall(_: &Context) -> cu::Result<()> {
    cu::bail!("cannot uninstall configuration for desktop environment");
}
