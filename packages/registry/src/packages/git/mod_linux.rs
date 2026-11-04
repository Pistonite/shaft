//! Git version control System. Uses Microsoft fork on Ubuntu

use crate::pre::*;

mod version;

register_binaries!("git", "scalar");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    if opfs::is_pacman() {
        check_pacman!("git");
        version::verify(false)
    } else {
        check_in_path!("git");
        version::verify(true)
    }
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    opfs::ensure_terminated("git")?;
    epkg::pacman::install_many(
        &["pcre2", "git"],
        "install git and dependencies",
        ctx.bar_ref(),
    )?;
    Ok(())
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    opfs::ensure_terminated("git")?;
    epkg::pacman::uninstall("git", ctx.bar_ref())?;
    Ok(())
}
