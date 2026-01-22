//! Pseudo package for the package manager itself

use crate::pre::*;

register_binaries!("sudo", "cargo");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    corelib::check_requirements()?;
    Ok(Verified::UpToDate)
}

pub fn install(_: &Context) -> cu::Result<()> {
    corelib::check_requirements()
}

pub fn config_location(_: &Context) -> cu::Result<Option<PathBuf>> {
    Ok(Some(hmgr::paths::config_toml()))
}

pub fn pre_uninstall(_: &Context) -> cu::Result<()> {
    cu::hint!("core is a pseudo package to check requirements of the tool itself, and cannot be uninstalled.");
    cu::bail!("cannot uninstall core");
}
pub use pre_uninstall as uninstall;
