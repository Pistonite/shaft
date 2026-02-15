//! Custom Desktop Environment via Hyprland

use crate::pre::*;

version_cache!(static CFG_VERSION = metadata::hyprland::CFG_VERSION);
binary_dependencies!(Viopen);
register_binaries!("vihypr");

mod sddm;

static PACKAGES: &[&str] = &[
    // fonts, login, and terminal
    "noto-fonts",
    "noto-fonts-emoji",
    "noto-fonts-cjk",
    "noto-fonts-extra",
    "ttf-hack-nerd",
    "sddm",
    "hyprland",
    "kitty",
    // audio stuff
    "pipewire",
    "wireplumber",
    "pipewire-pulse",
    "pipewire-jack",
    // video stuff
    "xdg-desktop-portal-hyprland",
    // DE stuff
    "qt5-wayland",
    "qt6-wayland",
    "polkit",
    "polkit-gnome", // authentication agent (password prompt)
    "hyprpaper", // wall paper
    "hyprlock", // lock screen
    "waybar", // status bar
    "rofi", // menu
    "networkmanager-dmenu", // wifi settings
    "swaync", // notification
    "cliphist", // clipboard
    "nautilus", // file manager
];

pub fn verify(_: &Context) -> cu::Result<Verified> {
    let v = check_pacman!("hyprland");
    check_outdated!(&v, metadata[hyprland]::VERSION);

    for package in PACKAGES {
        let v = epkg::pacman::installed_version(package)?;
        if v.is_none() {
            cu::error!("[cde-hyprland] {package} is not installed");
            return Ok(Verified::NotInstalled);
        }
    }

    check_version_cache!(CFG_VERSION);
    Ok(Verified::UpToDate)
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::pacman::install_many(PACKAGES, ctx.bar_ref())?;
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    cu::bail!("cannot uninstall desktop environment");
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let config = ctx.load_config(CONFIG)?;
    cu::check!(sddm::configure(&config.sddm), "failed to configure sddm")?;

    if let Some(mut home) = std::env::home_dir() {
        home.extend([".config", "hypr"]);
        ctx.add_item(Item::shim_bin(
            "vihypr",
            ShimCommand::target("viopen").args([home.into_utf8()?]),
        ))?;
    }
    ctx.add_item(Item::bash(r#"
explorer() {
    setsid nautilus "${1:-$HOME}" > /dev/null 2>&1 < /dev/null &
}
    "#))?;

    CFG_VERSION.update()?;

    Ok(())
}

config_file! {
    static CONFIG: Config = {
        template: include_str!("config.toml"),
        migration: []
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Config {
    sddm: sddm::SddmConfig
}

