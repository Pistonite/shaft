function main(config) {
    const META = config.meta;
    config = config.config;
    const KEY_BINDINGS = [
        { "id": null, "keys": "ctrl+v" },
        { "id": "User.find", "keys": "ctrl+shift+f" },
        { "id": "Terminal.MoveFocusDown", "keys": "ctrl+alt+j" },
        { "id": "User.copy.644BA8F2", "keys": "ctrl+c" },
        { "id": null, "keys": "ctrl+shift+d" },
        { "id": "Terminal.MoveFocusLeft", "keys": "ctrl+alt+h" },
        { "id": null, "keys": "alt+left" },
        { "id": "Terminal.MoveFocusRight", "keys": "ctrl+alt+l" },
        { "id": "Terminal.MoveFocusUp", "keys": "ctrl+alt+k" },
        { "id": null, "keys": "alt+down" },
        { "id": null, "keys": "alt+right" },
        { "id": "Terminal.MoveFocusPrevious", "keys": "ctrl+alt+6" },
        { "id": null, "keys": "ctrl+alt+left" },
        { "id": "Terminal.DuplicateTab", "keys": "ctrl+alt+w" },
        { "id": null, "keys": "alt+up" },
        { "id": null, "keys": "ctrl+shift+w" }
    ];
    const SCHEME = {
        "name": "Catppuccin Mocha/Frappe",
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
    };
    const THEME_NAME = "Catppuccin Mocha"
    const THEMES = [ {
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
    }, {
        "name": THEME_NAME,
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
    } ];
    const HIDE_PROFILE_SOURCES = [
        "Windows.Terminal.VisualStudio",
        "Windows.Terminal.Azure",
    ];
    const POWERSHELL5_PROFILE = {
        "commandline": "%SystemRoot%\\System32\\WindowsPowerShell\\v1.0\\powershell.exe -NoLogo",
        "guid": "{61c54bbd-c2c6-5271-96e7-009a87ff44bf}",
        "hidden": false,
        "name": "Windows PowerShell"
    };
    const POWERSHELL7_PROFILE = {
        "commandline": META.install_dir + "\\pwsh.exe -NoLogo",
        "guid": "{bb6f7902-320e-4f8c-bbad-9578445057d2}",
        "hidden": false,
        "icon": META.install_dir + "\\pwsh.exe",
        "name": "PowerShell 7",
        "startingDirectory": "%USERPROFILE%"
    };

    if (!is_object(config)) {
        config = {};
    }
    ensure_object(config, "profiles");
    ensure_object(config.profiles, "defaults");
    ensure_object(config.profiles.defaults, "font");
    ensure_array(config.profiles, "list");
    ensure_array(config, "schemes");
    ensure_array(config, "themes");

    function zap_schemes(config) {
        let index = config.schemes.length;
        for (let i=0;i<index;i++) {
            if (config.schemes[i].name === SCHEME.name) {
                index = i; break;
            }
        }
        if (index === config.schemes.length) {
            config.schemes.push(SCHEME);
        } else {
            config.schemes[index] = SCHEME;
        }
        config.profiles.defaults.colorScheme = SCHEME.name;
    }
    zap_schemes(config);

    function zap_theme(config, theme) {
        let index = config.themes.length;
        for (let i=0;i<index;i++) {
            if (config.themes[i].name === theme.name) {
                index = i; break;
            }
        }
        if (index === config.themes.length) {
            config.themes.push(theme);
        } else {
            config.themes[index] = theme;
        }
    }
    for (const theme of THEMES) {
        zap_theme(config, theme);
    }
    config.theme = THEME_NAME;
    config.keybindings = KEY_BINDINGS;
    config.profiles.defaults.font.face = "Hack Nerd Font";
    config.profiles.defaults.adjustIndistinguishableColors = "indexed";
    config.profiles.defaults.cursorShape = "bar";
    config.profiles.defaults.intenseTextStyle = "bright";
    config.profiles.defaults.padding = "2";
    config.profiles.defaults.scrollbarState = "hidden";

    function find_ps_profiles() {
        let ps7_index = -1;
        let ps5_index = -1;
        const profile_len = config.profiles.list.length;
        for (let i=0;i<profile_len;i++) {
            const profile = config.profiles.list[i];
            if (HIDE_PROFILE_SOURCES.includes(profile.source)) {
                profile.hidden = true;
            }
            if (profile.name === "PowerShell 7") {
                ps7_index = i;
            } else if (profile.name === "Windows PowerShell") {
                ps5_index = i;
            }
        }
        return [ps5_index, ps7_index];
    }

    if (META.pwsh_installed) {
        config.defaultProfile = POWERSHELL7_PROFILE.guid;
        let [ps5_index, ps7_index] = find_ps_profiles();
        if (ps5_index === 1 && ps7_index === 0) {
            config.profiles.list[ps5_index] = POWERSHELL5_PROFILE;
            config.profiles.list[ps7_index] = POWERSHELL7_PROFILE;
        } else {
            while (ps5_index !== -1) {
                config.profiles.list.splice(ps5_index, 1);
                [ps5_index, ps7_index] = find_ps_profiles();
            }
            while (ps7_index !== -1) {
                config.profiles.list.splice(ps7_index, 1);
                [ps5_index, ps7_index] = find_ps_profiles();
            }
            config.profiles.list.splice(0, 0, POWERSHELL7_PROFILE, POWERSHELL5_PROFILE);
        }
    } else {
        config.defaultProfile = POWERSHELL5_PROFILE.guid;
        let [ps5_index, ps7_index] = find_ps_profiles();
        if (ps5_index === 0 && ps7_index === -1) {
            config.profiles.list[ps5_index] = POWERSHELL5_PROFILE;
        } else {
            while (ps5_index !== -1) {
                config.profiles.list.splice(ps5_index, 1);
                [ps5_index, ps7_index] = find_ps_profiles();
            }
            while (ps7_index !== -1) {
                config.profiles.list.splice(ps7_index, 1);
                [ps5_index, ps7_index] = find_ps_profiles();
            }
            config.profiles.list.splice(0, 0, POWERSHELL5_PROFILE);
        }
    }

    return config;
}
