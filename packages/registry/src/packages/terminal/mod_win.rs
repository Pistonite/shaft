//! Configuration for Windows Terminal

use crate::pre::*;

static INTERNAL_VERSION: &str = "1";

pub fn config_dependencies() -> EnumSet<PkgId> {
    Default::default()
}

pub fn verify(ctx: &Context) -> cu::Result<Verified> {
    let setting_json = setting_json()?;
    if !setting_json.exists() {
        return Ok(Verified::NotInstalled);
    }
    let id = ctx.pkg.to_str();
    let version = hmgr::get_cached_version(id)?;
    Ok(Verified::is_uptodate(version.as_deref() == Some(INTERNAL_VERSION)))
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    cu::warn!("please also install/update HackNerdFont:\n  https://github.com/ryanoasis/nerd-fonts/releases");
    let _ = cu::prompt!("press ENTER when confirmed HackNerdFont is installed")?;
    cu::check!(verify(ctx), "system-git requires 'git' to be installed on the system")?;
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    Ok(())
}

#[cu::context("failed to get windows terminal settings dir")]
fn setting_dir() -> cu::Result<PathBuf> {
    let mut p = PathBuf::from(cu::env_var("LOCALAPPDATA")?);
    p.extend(["Packages", "Microsoft.WindowsTerminal_8wekyb3d8bbwe", "LocalState"]);
    Ok(p)
}

fn setting_json() -> cu::Result<PathBuf> {
    let mut p = setting_dir()?;
    p.push("settings.json");
    Ok(p)
}

fn zap_setting_json(config: &mut json::Value) {
    if !config.is_object() {
        *config = json!({});
    }
    config["showTabsInTitlebar"] = false.into();
    config["alwaysShowTabs"] = false.into();
    zap_schemes(config);
    zap_key_bindings(config);
    zap_font(config);
    zap_themes(config);
    let profiles_defaults = get_profiles_defaults_mut(config);
    profiles_defaults["adjustIndistinguishableColors"] = "always".into();
    profiles_defaults["cursorShape"] = "bar".into();
    profiles_defaults["intenseTextStyle"] = "bright".into();
    profiles_defaults["padding"] = "2".into();
    profiles_defaults["scrollbarState"] = "hidden".into();
}

fn zap_schemes(config: &mut json::Value) {
    let scheme_name = "Catppuccin Mocha/Frappe";

    let schemes = match config.get_mut("schemes") {
        Some(x) if x.is_array() => x.as_array_mut().unwrap(),
        _ => {
            config["schemes"] = json!([]);
            config.get_mut("schemes").unwrap().as_array_mut().unwrap()
        }
    };
    let mut the_scheme = None;
    for scheme in schemes.iter_mut() {
        let Some(name) = scheme.get_mut("name") else {
            continue;
        };
        if name.as_str() != Some(scheme_name) {
            continue;
        }
        the_scheme = Some(scheme);
        break;
    }
    let new_scheme = json!{
        {
            "name": scheme_name,
            "background": "#303446",
            "black": "#45475A",
            "blue": "#89B4FA",
            "brightBlack": "#585B70",
            "brightBlue": "#89B4FA",
            "brightCyan": "#94E2D5",
            "brightGreen": "#A6E3A1",
            "brightPurple": "#F5C2E7",
            "brightRed": "#F38BA8",
            "brightWhite": "#A6ADC8",
            "brightYellow": "#F9E2AF",
            "cursorColor": "#F5E0DC",
            "cyan": "#94E2D5",
            "foreground": "#CDD6F4",
            "green": "#A6E3A1",
            "purple": "#F5C2E7",
            "red": "#F38BA8",
            "selectionBackground": "#585B70",
            "white": "#BAC2DE",
            "yellow": "#F9E2AF" 
        }
    };
    match the_scheme {
        None => schemes.push(new_scheme),
        Some(x) => *x = new_scheme
    }

    let profiles_defaults = get_profiles_defaults_mut(config);
    profiles_defaults["colorScheme"] = scheme_name.into();
}

fn zap_key_bindings(config: &mut json::Value) {
    config["keybindings"] = json!{
        [
            {
                "id": null,
                "keys": "ctrl+v"
            },
            {
                "id": "User.find",
                "keys": "ctrl+shift+f"
            },
            {
                "id": "Terminal.MoveFocusDown",
                "keys": "ctrl+alt+j"
            },
            {
                "id": "User.copy.644BA8F2",
                "keys": "ctrl+c"
            },
            {
                "id": null,
                "keys": "ctrl+shift+d"
            },
            {
                "id": "Terminal.MoveFocusLeft",
                "keys": "ctrl+alt+h"
            },
            {
                "id": null,
                "keys": "alt+left"
            },
            {
                "id": "Terminal.MoveFocusRight",
                "keys": "ctrl+alt+l"
            },
            {
                "id": "Terminal.MoveFocusUp",
                "keys": "ctrl+alt+k"
            },
            {
                "id": null,
                "keys": "alt+down"
            },
            {
                "id": null,
                "keys": "alt+right"
            },
            {
                "id": "Terminal.MoveFocusPrevious",
                "keys": "ctrl+alt+6"
            },
            {
                "id": null,
                "keys": "ctrl+alt+left"
            },
            {
                "id": "Terminal.DuplicateTab",
                "keys": "ctrl+alt+w"
            },
            {
                "id": null,
                "keys": "alt+up"
            },
            {
                "id": null,
                "keys": "ctrl+shift+w"
            }
        ]
        };
}

fn zap_font(config: &mut json::Value) {
    let profiles_defaults = get_profiles_defaults_mut(config);
    match profiles_defaults.get_mut("font") {
        Some(x) if x.is_object() => {
            x["face"] = "Hack Nerd Font".into();
        }
        _ => {
            profiles_defaults["font"] = json!{{
                "face": "Hack Nerd Font"
            }};
        }
    };
}

fn zap_themes(config: &mut json::Value) {
    let themes = match config.get_mut("themes") {
        Some(x) if x.is_array() => x.as_array_mut().unwrap(),
        _ => {
            config["themes"] = json!([]);
            config.get_mut("themes").unwrap().as_array_mut().unwrap()
        }
    };

    let mut latte = None;
    let mut mocha = None;
    for theme in themes.iter_mut() {
        let Some(name) = theme.get_mut("name") else {
            continue;
        };
        let Some(name) = name.as_str() else {
            continue;
        };
        match name {
            "Catppuccin Latte" => latte = Some(theme),
            "Catppuccin Mocha" => mocha = Some(theme),
            _ => {}
        }
    }
    let mut extra = vec![];
    let new_latte = json!{
        {
            "name": "Catppuccin Latte",
            "tab": 
            {
                "background": "#EFF1F5FF",
                "iconStyle": "default",
                "showCloseButton": "always",
                "unfocusedBackground": null
            },
            "tabRow": 
            {
                "background": "#E6E9EFFF",
                "unfocusedBackground": "#DCE0E8FF"
            },
            "window": 
            {
                "applicationTheme": "light",
                "experimental.rainbowFrame": false,
                "frame": null,
                "unfocusedFrame": null,
                "useMica": false
            }
        }
    };
    let new_mocha = json!{
        {
            "name": "Catppuccin Mocha",
            "tab": 
            {
                "background": "#1E1E2EFF",
                "iconStyle": "default",
                "showCloseButton": "always",
                "unfocusedBackground": null
            },
            "tabRow": 
            {
                "background": "#181825FF",
                "unfocusedBackground": "#11111BFF"
            },
            "window": 
            {
                "applicationTheme": "dark",
                "experimental.rainbowFrame": false,
                "frame": null,
                "unfocusedFrame": null,
                "useMica": false
            }
        }
    };
    match latte {
        Some(x) => *x = new_latte,
        None => extra.push(new_latte)
    }
    match mocha {
        Some(x) => *x = new_mocha,
        None => extra.push(new_mocha)
    }
    themes.extend(extra);
    config["theme"] = "Catppuccin Mocha".into();
}

fn get_profiles_defaults_mut(config: &mut json::Value) -> &mut json::Value {
    match config.get_mut("profiles") {
        Some(x) if x.is_object() => {}
        _ => {
            config["profiles"] = json!({});
        }
    };
    match config.get_mut("profiles").unwrap().get_mut("defaults") {
        Some(x) if x.is_object() => {
        }
        _ => {
            config["profiles"]["defaults"] = json!({});
        }
    }
    config.get_mut("profiles").unwrap().get_mut("defaults").unwrap()
}
