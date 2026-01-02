//! Tool for installing cargo tools from binary releases

use op::Version;

use cu::pre::*;

use crate::pre::*;

register_binaries!("cargo-binstall");
static VERSION: &str = "1.16.6";

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_bin_in_path!("cargo-binstall");
    match op::installer::cargo::installed_info("cargo-binstall")? {
        None => return Ok(Verified::NotInstalled),
        Some(info) => {
            if Version(&info.version) < VERSION {
                return Ok(Verified::NotUpToDate);
            }
        }
    }
    let current_version = op::command_output!("cargo-binstall", ["-V"]);
    if Version(&current_version) < VERSION {
        return Ok(Verified::NotUpToDate);
    }

    Ok(Verified::UpToDate)
}

pub fn install(_: &Context) -> cu::Result<()> {
    if cu::which("cargo-binstall").is_ok() {
        op::installer::cargo::binstall("cargo-binstall")
    } else {
        op::installer::cargo::install("cargo-binstall")
    }
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    op::installer::cargo::uninstall("cargo-binstall")
}
