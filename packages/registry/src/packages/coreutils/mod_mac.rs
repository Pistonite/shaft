//! Extra Coreutils for Mac (does not override BSD-derived versions that come with MacOS)

use crate::pre::*;

mod common;
mod eza;

register_binaries!("ls", "la");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_verified!(eza::verify()?);

    check_config_version_cache!(common::ALIAS_VERSION);
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    eza::install(ctx)?;
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    eza::configure(ctx)?;

    // using shell alias for UI-only differences
    let grep_alias = "alias grep='grep --color=auto'";
    ctx.add_item(Item::bash(grep_alias))?;
    ctx.add_item(Item::zsh(grep_alias))?;
    common::ALIAS_VERSION.update()?;
    Ok(())
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    eza::uninstall(ctx)?;
    Ok(())
}
