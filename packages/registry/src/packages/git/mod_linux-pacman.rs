//! Git version control System

use crate::pre::*;

mod version;

register_binaries!("git");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_pacman!("git");
    version::verify(false)
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
