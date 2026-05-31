//! Configuration for Framework Laptop 16
use crate::pre::*;

version_cache!(static KBD_ULEDS_VERSION = metadata::framework16::kbd_uleds::COMMIT);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_pacman!("fw16-kbd-uleds-git");
    check_config_version_cache!(KBD_ULEDS_VERSION);
    Ok(Verified::UpToDate)
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    let install_dir = ctx.install_dir();
    epkg::pacman::install_aur(
        "fw16-kbd-uleds-git",
        "https://aur.archlinux.org/fw16-kbd-uleds-git.git",
        &install_dir,
        ctx.bar_ref(),
    )?;
    KBD_ULEDS_VERSION.update()?;
    Ok(())
}
pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    epkg::pacman::uninstall("fw16-kbd-uleds-git", ctx.bar_ref())?;
    Ok(())
}
