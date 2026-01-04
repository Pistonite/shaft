//! Git version control System

use crate::pre::*;

pub mod version;

register_binaries!("git");

static GIT_VERSION: &str = "2.51.2";

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_bin_in_path!("git");
    check_installed_with_pacman!("git", "system-git");
    version::verify(GIT_VERSION)
}

pub fn install(_: &Context) -> cu::Result<()> {
    opfs::ensure_terminated("git")?;
    epkg::pacman::install("git")?;
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    opfs::ensure_terminated("git")?;
    epkg::pacman::uninstall("git")?;
    Ok(())
}

