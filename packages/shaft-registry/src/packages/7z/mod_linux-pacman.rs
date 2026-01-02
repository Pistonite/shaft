use crate::pre::*;

mod version;

register_binaries!("7z");

static VERSION: &str = "25.01";

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_bin_in_path!("7z");
    check_installed_with_pacman!("7zip");
    version::check(VERSION)
}
pub fn install(_: &Context) -> cu::Result<()> {
    op::sysinfo::ensure_terminated("7z")?;
    op::installer::pacman::install("7zip")?;
    Ok(())
}
pub fn uninstall(_: &Context) -> cu::Result<()> {
    op::installer::pacman::uninstall("7zip")?;
    Ok(())
}
