use crate::pre::*;
version_cache!(pub static ALIAS_VERSION = metadata::coreutils::ALIAS_VERSION);

pub fn linux_configure(ctx: &Context) -> cu::Result<()> {
    // using shell alias for UI-only differences
    let grep_alias = "alias grep='grep --color=auto'";
    ctx.add_item(Item::bash(grep_alias))?;
    ctx.add_item(Item::zsh(grep_alias))?;
    Ok(())
}
