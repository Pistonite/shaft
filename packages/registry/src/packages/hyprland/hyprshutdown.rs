use crate::pre::*;

pub fn verify() -> cu::Result<Verified> {
    let v = check_pacman!("hyprshutdown");
    check_outdated!(&v, metadata[hyprland]::VERSION);
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
if     let Ok(Verified::UpToDate) = verify() && cu::which("hyprshutdown").is_ok() {
        return Ok(());
    }
    let install_dir = ctx.install_dir();
    epkg::pacman::install_aur("hyprshutdown", 
        "https://aur.archlinux.org/hyprshutdown.git", &install_dir, ctx.bar_ref())?;
    let install_dir = install_dir.join("hyprshutdown");
    cu::fs::write(install_dir.join("hyprshutdownw"), HYPRSHUTDOWNW)?;
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let mut path = ctx.install_dir();
    path.extend(["hyprshutdown", "hyprshutdownw"]);
    ctx.add_item(Item::link_bin("hyprshutdownw", path.into_utf8()?))?;
    Ok(())
}

/// wrapper: hyprshutdownw <command>...
/// will just invoke command if hyprshutdown is not found
static HYPRSHUTDOWNW: &str = r##"#!/usr/bin/env bash

cmd="$*"
if command -v hyprshutdown >/dev/null 2>&1; then
    hyprshutdown -p "$cmd"
else
    eval "$cmd"
fi
"##;
