//! Microsoft fork of Git

use crate::pre::*;

register_binaries!("git", "scalar");

static VERSION: Version = Version("2.52.0.vfs");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_bin_in_path!("git");
    let version = command_output!("git", ["--version"]);
    if !version.contains("vfs") {
        cu::bail!("current 'git' is not the vfs version (microsoft.git); please uninstall it or use the 'system-git' package");
    }
    check_bin_in_path!("scalar");
    let version = version.strip_prefix("git version").unwrap_or(&version);
    let is_uptodate = VERSION <= version.trim();
    Ok(Verified::is_uptodate(is_uptodate))
}

pub fn install(_: &Context) -> cu::Result<()> {
    opfs::ensure_terminated("git.exe")?;
    opfs::ensure_terminated("scalar.exe")?;
    epkg::winget::install("Microsoft.Git")?;
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    opfs::ensure_terminated("git.exe")?;
    opfs::ensure_terminated("scalar.exe")?;
    epkg::winget::uninstall("Microsoft.Git")?;
    Ok(())
}
