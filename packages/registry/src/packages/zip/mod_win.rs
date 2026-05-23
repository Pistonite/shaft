//! Zip and Unzip commands
use crate::pre::*;

register_binaries!(
    "zip",
    "unzip"
);
pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_in_shaft!("zip");
    check_in_shaft!("unzip");
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    // copy zip for windows from registry
    // note that GnuWin for unzip is outdated
    let mut zip_path = hmgr::paths::repo();
    zip_path.extend(["packages", "registry", "src", "packages", "zip", "zip-win"]);

    let install_path = ctx.install_dir();

}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    Ok(())
}
