//! Configuration for Terminal

use crate::pre::*;

pub fn verify(_: &Context) -> cu::Result<Verified> {
    let version = check_pacman!("ttf-hack-nerd");
    check_outdated!(version.trim(), metadata[hack_font]::VERSION_PACMAN);
    Ok(Verified::UpToDate)
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::pacman::install("ttf-hack-nerd", ctx.bar_ref())
}
pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    epkg::pacman::uninstall("ttf-hack-nerd", ctx.bar_ref())
}
