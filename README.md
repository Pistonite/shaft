# shaft

`shaft` is my package and config manager that allows me to set up
tools and environments consistently cross-platform.

It serves as my: 
- "dotfiles" repo i.e. software configs, but cross-platform.
- Installation scripts
- Version tracker
- Utility scripts
- Setup documentation

## Requirements
- Windows:
  - Sudo for Windows: [How to enable](https://learn.microsoft.com/en-us/windows/advanced-settings/sudo).
  - Set up a dev drive for optimal performance: [How to set up](https://learn.microsoft.com/en-us/windows/dev-drive/).
  - Windows PowerShell (AKA PowerShell 5) should be included with Windows.
    Note that not all packages will work in CMD, but most non-user-interaction type software
    will. For example, `zoxide` does not support CMD.
  - Install [Rust](https://rustup.rs) toolchain and MSVC.
- Arch Linux:
  - `sudo` or `sudo-rs` package should be installed when bootstrapping the system.
  - Only `bash` is supported for now as that's what I use.
  - Install [Rust](https://rustup.rs) toolchain.

## Install
You can download the binary from the release page or build it from
source with `cargo install shaft-cli --git https://github.com/Pistonite/shaft --locked`.
When upgrading with the `shaft` command, it will download from github releases
using `cargo-binstall`.

Once downloaded, run the binary without any arguments in Windows PowerShell on windows
or any terminal on Non-Windows to begin the setup steps:

### Windows
1. Run the downloaded/installed binary in Windows PowerShell.
1. Setup `SHAFT_HOME`. This will let you pick a location for `shaft`
   to store everything. The default for Windows is `%USERPROFILE%/.config/pistonite-shaft`.
   If you use a dev drive, set the path somewhere on the dev drive, like `X:/.config/pistonite-shaft`
   where `X` is the drive label for the dev drive.
1. The setup will add `SHAFT_HOME` to the current user's environment variables.
   It will also modify the current user's `PATH` to include everything installed by `shaft`.
1. Add the `shaft` init script to Windows PowerShell (AKA PowerShell 5)'s `$PROFILE.CurrentUserAllHosts`.
   ```powershell
   # Shaft init script
   . $env:SHAFT_HOME\init.pwsh
   ```
   Note that PowerShell 7 is not included in Windows. It's installed with the `windows-pwsh` package in `shaft`.
   `shaft` will manage all configurations for PowerShell 7.
1. Restart Windows PowerShell.

### Non-Windows
1. Run the downloaded/installed binary in a terminal.
1. Setup `SHAFT_HOME`. This will let you pick a location for `shaft`
   to store everything. The default for Non-Windows is `~/.config/pistonite-shaft`.
1. Modify your shell profile (for example `~/.bashrc`) to source the following file:
   ```bash
   # Shaft init script
   . ~/.config/pistonite-shaft/init.bash 
   ```
   Replace the path with the correct path to the installation. The init script
   will set up the `SHAFT_HOME` and `PATH` environment variables.
   You have to use the full path when you source the file.
1. Restart the terminal.

## Upgrade
`shaft` will track the compatible versions of installed software. This "registry"
is compiled directly into the binary. Upgrading the registry is the same as upgrading
the binary. When a command is run that installs/removes package(s), a version check
will run to see if a newer version is available on GitHub.

The versioning scheme only uses `MINOR` and `PATCH` versions. The `MAJOR` version is always 0.
- `PATCH` - no change to registry. There will be a hint but no prompt for upgrading.
- `MINOR` - registry is changed. There will be a prompt for upgrading.

You can run `ds upgrade` to upgrade with `cargo-binstall`.
Note that `cargo-binstall` is installed with the `platform-utils` package.
If it's not installed, `cargo install` will be used instead.

If a package is removed from the registry (due to malware or something),
the package must be uninstalled before upgrading.

## Commands

| Command | Description |
|-|-|
| `ds sync PACKAGE...` | Sync (install) packages. If packages aren't provided. It will verify and resync the install cache. |
| `ds remove PACKAGE...` | Uninstall packages. Note that packages depended on by others *can* be uninstalled after confirmation. (For example, uninstalling `git` to install `microsoft-git`) |
| `ds info PACKAGE_OR_BIN` | Fuzzy search package/binary |
| `ds config PACKAGE` | Open package TOML config in a text editor |
| `ds upgrade` | Upgrade the `shaft` binary and registry |
