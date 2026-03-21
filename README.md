# shaft

`shaft` is my package and config manager that allows me to set up
the same tools and configs in multiple environments and cross-platform.

It serves as my: 
- "dotfiles" repo i.e. software configs, but cross-platform.
- Installation scripts
- Version tracker
- Utility scripts
- Setup documentation

Documentation below is for me to setup `shaft` on a new machine.

## Requirements
- Windows:
  - Sudo for Windows: [How to enable](https://learn.microsoft.com/en-us/windows/advanced-settings/sudo).
  - Set up a dev drive for optimal performance: [How to set up](https://learn.microsoft.com/en-us/windows/dev-drive/).
  - Install [Rust](https://rustup.rs) toolchain and MSVC.
- Other:
  - `sudo`
  - Install [Rust](https://rustup.rs) toolchain and build tools.

## Install/Upgrade
Installing for the first time: clone and build
```
git clone https://github.com/Pistonite/shaft
cd shaft
cargo run --bin shaft-build
cargo run --bin shaft
```

After setup is complete:
```
cargo run --bin shaft upgrade
```

To upgrade run `shaft upgrade`

## ARM build issue
For ARM64, if build is failing, consider using the X64 emulation mode.

First add the target (replace the target triple with the one for your OS)
```
rustup target add x86_64-pc-windows-msvc
```
Then replace `cargo run --bin shaft` above with
```
cargo run --bin shaft --target x86_64-pc-windows-msvc --features build-x64
```
