//! Configuration documentation for Windows OS

use crate::pre::*;
pub fn verify(ctx: &Context) -> cu::Result<Verified> {
    if !ctx.is_installed(PkgId::WindowsCfg) {
        cu::hint!(
            "windows-cfg is a pseudo package for documenting Windows configuration, use `shaft config windows-cfg` to read"
        );
    }
    Ok(Verified::UpToDate)
}

pub fn install(_: &Context) -> cu::Result<()> {
    Ok(())
}

pub fn config_location(ctx: &Context) -> cu::Result<Option<PathBuf>> {
    let path = ctx.config_file();
    cu::fs::write(&path, include_str!("config.toml"))?;
    Ok(Some(path))
}

pub fn pre_uninstall(_: &Context) -> cu::Result<()> {
    cu::hint!("windows-cfg is a pseudo package for documenting Windows configuration");
    cu::bail!("cannot uninstall windows-cfg");
}
pub use pre_uninstall as uninstall;
