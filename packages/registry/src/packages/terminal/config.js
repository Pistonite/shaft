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
        "Git",
    ];
    const CMD_GUID = "{0caa0dad-35be-5f56-a8ff-afceeeaa6101}";
    const HIDE_PROFILE_GUIDS = [
        CMD_GUID
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
    const CLINK_CMD_GUID = "{50a056c9-bbde-4012-82c7-b5215e03254f}";
    const CLINK_CMD_PROFILE = {
        "commandline": META.clink_cmd_bin,
        "guid": CLINK_CMD_GUID,
        "hidden": false,
        "icon": "%SystemRoot%\\System32\\cmd.exe",
        "name": "Command Prompt",
        "startingDirectory": "%USERPROFILE%"
    };
    const CMD_EXEs = [
        "%comspec%",
        "%systemroot%\\system32\\cmd.exe",
        "%windir%\\system32\\cmd.exe",
        "%systemdrive%\\windows\\system32\\cmd.exe",
        META.cmd_bin.toLowerCase(),
    ];

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

    function zap_profiles(config) {
        function parse_lpcommandline(xxx) {
            if (xxx.trim().startsWith('"')) {
                const end_i = xxx.indexOf('"', 1);
                if (end_i === -1) {
                    return [xxx.substring(1), ""];
                }
                return [xxx.substring(1, end_i), xxx.substring(end_i+1)];
            }
            const space_i = xxx.indexOf(' ');
            if (space_i === -1) {
                return [xxx, ""];
            }
            return [xxx.substring(0, space_i), xxx.substring(space_i)];
        }
        const non_controlled_profiles = [];
        const profile_len = config.profiles.list.length;
        for (let i=0;i<profile_len;i++) {
            const profile = config.profiles.list[i];
            if (HIDE_PROFILE_SOURCES.includes(profile.source)) {
                profile.hidden = true;
            } else if (HIDE_PROFILE_GUIDS.includes(profile.guid)) {
                profile.hidden = true;
            }
            if (profile.name === "PowerShell 7") {
                continue;
            } else if (profile.name === "Windows PowerShell") {
                continue;
            } else if (profile.guid === CLINK_CMD_GUID) {
                continue;
            }

            // convert cmd to clink_cmd
            const commandline = profile.commandline;
            if (commandline?.trim()) {
                const [executable, rest] = parse_lpcommandline(commandline);
                if (CMD_EXEs.includes(executable.toLowerCase())) {
                    if (META.clink_cmd_bin.includes(' ')) {
                        profile.commandline = `"${META.clink_cmd_bin}"${rest}`;
                    } else {
                        profile.commandline = `${META.clink_cmd_bin} ${rest}`;
                    }
                }
            }

            non_controlled_profiles.push(profile);
        }
        config.profiles.list = [];
        if (META.pwsh_installed) {
            config.defaultProfile = POWERSHELL7_PROFILE.guid;
            config.profiles.list.push(POWERSHELL7_PROFILE)
        } else {
            config.defaultProfile = POWERSHELL5_PROFILE.guid;
        }
        config.profiles.list.push(POWERSHELL5_PROFILE, CLINK_CMD_PROFILE);
        config.profiles.list.push(...non_controlled_profiles);
    }
    zap_profiles(config);
    config.showTabsInTitlebar=false;
    config.alwaysShowTabs=false;

    return config;
}
