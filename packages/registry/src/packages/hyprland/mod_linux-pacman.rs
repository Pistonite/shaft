//! Custom Hyprland Desktop Environment

use crate::pre::*;

version_cache!(static CFG_VERSION = metadata::hyprland::CFG_VERSION);
binary_dependencies!(
    Viopen, 
    Python,
    // build dependencies
    Cmake,
    Gcc,
    Git,
    Ninja,
);
// config_dependencies!(Framework16);
register_binaries!("explorer", "vihypr");

mod sddm;
mod hyprshutdown;

static PACKAGES: &[&str] = &[
    // fonts, login, and terminal
    "noto-fonts",
    "noto-fonts-emoji",
    "noto-fonts-cjk",
    "noto-fonts-extra",
    "ttf-hack-nerd",
    "sddm",
    "hyprland",
    "kitty", // ~/.config/kitty/kitty.conf
    // audio/video/hardware stuff
    "pipewire",
    "wireplumber",
    "pipewire-pulse",
    "pipewire-jack",
    "xdg-desktop-portal-hyprland",
    "brightnessctl",
    // DE stuff
    "hyprpaper", // wall paper
    "hyprlock", // lock screen
    "hypridle", // idle management
    "qt5-wayland",
    "qt6-wayland",
    "polkit",
    "polkit-gnome", // authentication agent (password prompt)
    "waybar", // status bar ~/.config/waybar
    "rofi", // menu ~/.config/rofi/config.rasi
    "networkmanager-dmenu", // wifi settings
    "swaync", // notification
    "cliphist", // clipboard
    "nautilus", // file manager
    //
    // TODO: https://github.com/paco3346/fw16-kbd-uleds
];

pub fn verify(_: &Context) -> cu::Result<Verified> {
    let v = check_pacman!("hyprland");
    check_outdated!(&v, metadata[hyprland]::VERSION);

    for package in PACKAGES {
        let v = epkg::pacman::installed_version(package)?;
        if v.is_none() {
            cu::error!("[hyprland] {package} is not installed");
            return Ok(Verified::NotInstalled);
        }
    }
    check_verified!(hyprshutdown::verify()?);

    check_version_cache!(CFG_VERSION);
    Ok(Verified::UpToDate)
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::pacman::install_many(PACKAGES, ctx.bar_ref())?;
    hyprshutdown::install(ctx)?;
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    cu::bail!("cannot uninstall desktop environment");
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let config = ctx.load_config(CONFIG)?;
    cu::check!(sddm::configure(&config.sddm), "failed to configure sddm")?;
    cu::check!(hyprshutdown::configure(ctx), "failed to configure hyprshutdown")?;

    // CANDO: .config/hypr/hyprpaper.conf
    // TODO: hyprlock
    // TODO: .config/hypr
    // TODO: .config/waybar

    let home = cu::check!(std::env::home_dir(), "failed to get home directory")?;
    let config_dir = home.join(".config");
    let hypr_dir = config_dir.join("hypr");
        ctx.add_item(Item::shim_bin(
            "vihypr",
            ShimCommand::target("viopen").args([hypr_dir.as_utf8()?]),
        ))?;
    
    ctx.add_item(Item::shim_bin("explorer", ShimCommand::target("nautilus")))?;



    CFG_VERSION.update()?;

    Ok(())
}



config_file! {
    static CONFIG: Config = {
        template: include_str!("config.toml"),
        migration: [""]
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Config {
    sddm: sddm::SddmConfig,
    hyprland: HyprlandConfig,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct HyprlandConfig {
    monitor: HyprlandMonitorConfig
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct HyprlandMonitorConfig {
    name: String,
    mode: String,
    position: String,
    scale: String,
}

