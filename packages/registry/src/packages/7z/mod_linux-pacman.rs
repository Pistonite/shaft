//! 7-Zip
use crate::pre::*;

mod version;

register_binaries!("7z");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_bin_in_path!("7z");
    check_installed_with_pacman!("7zip");
    version::check(metadata::_7z::VERSION)
}
pub fn install(_: &Context) -> cu::Result<()> {
    opfs::ensure_terminated("7z")?;
    epkg::pacman::install("7zip")?;
    Ok(())
}
pub fn uninstall(_: &Context) -> cu::Result<()> {
    opfs::ensure_terminated("7z")?;
    epkg::pacman::uninstall("7zip")?;
    Ok(())
}
