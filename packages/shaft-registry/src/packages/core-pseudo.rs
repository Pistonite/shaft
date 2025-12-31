
//! Pseudo package for checking required dependencies for the package manager itself

use crate::pre::*;

metadata_binaries!("sudo", "cargo");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    if let Err(e) = cu::which("sudo") {
        cu::error!("sudo not found: {e:?}");
        if cfg!(windows) {
            cu::hint!("sudo is required; please install sudo with your system package manager.");
        } else {
            cu::hint!("sudo is required.");
            cu::hint!("please refer to the following link to enable it on Windows");
            cu::hint!("https://learn.microsoft.com/en-us/windows/advanced-settings/sudo");
        }
        cu::bail!("requirement not satisfied: sudo not found in PATH");
    }
    cu::debug!("sudo is found");
    if let Err(e) = cu::which("cargo") {
        cu::error!("cargo not found: {e:?}");
        cu::hint!("rust toolchain is required for sanctvm to work.");
        cu::hint!("please refer to: https://rustup.rs");
        if cfg!(windows) {
            cu::hint!("note that MSVC build tools also need to be installed on Windows.");
        }
        cu::bail!("requirement not satisfied: cargo not found in PATH");
    }
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    verify(ctx)?;
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    cu::hint!("core-pseudo is a pseudo package to check requirements of the tool itself, and cannot be uninstalled.");
    cu::bail!("cannot uninstall core-pseudo");
}
