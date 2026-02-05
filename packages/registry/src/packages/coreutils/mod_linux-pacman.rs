//! GNU Coreutils, Diffutils, and other basic commands

use crate::pre::*;

mod common;
mod eza;

register_binaries!(
    "ls",
    "diff",
    "find",
    "gzip",
    "sed",
    "grep",
    "zip",
    "unzip",
    "tar",
    "update-mirrors"
);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    eza::verify()?;
    check_in_shaft!("update-mirrors");
    check_pacman!("base");

    let v = check_pacman!("bash");
    check_outdated!(&v, metadata[coreutils::bash]::VERSION);
    let v = check_pacman!("bash-completion");
    check_outdated!(&v, metadata[coreutils::bash_cmp]::VERSION);

    let v = check_pacman!("zip");
    check_outdated!(&v, metadata[coreutils::zip]::VERSION);
    let v = check_pacman!("unzip");
    check_outdated!(&v, metadata[coreutils::unzip]::VERSION);
    let v = check_pacman!("tar");
    check_outdated!(&v, metadata[coreutils::tar]::VERSION);
    let v = check_pacman!("which");
    check_outdated!(&v, metadata[coreutils::which]::VERSION);

    check_version_cache!(common::ALIAS_VERSION);
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    eza::install(ctx)?;
    let install_dir = ctx.install_dir();
    let update_mirrors_sh = install_dir.join("update-mirrors.sh");
    cu::fs::write(
        update_mirrors_sh,
        include_bytes!("./pacman-update-mirrors.sh"),
    )?;

    epkg::pacman::install("base", ctx.bar_ref())?;
    epkg::pacman::install("bash-completion", ctx.bar_ref())?;
    epkg::pacman::install("which", ctx.bar_ref())?;
    epkg::pacman::install("zip", ctx.bar_ref())?;
    epkg::pacman::install("unzip", ctx.bar_ref())?;
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    eza::uninstall()?;
    cu::warn!("not uninstalling the essential packages for your sanity");
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    eza::configure(ctx)?;
    let install_dir = ctx.install_dir();
    let update_mirrors_sh = install_dir.join("update-mirrors.sh");

    ctx.add_item(Item::shim_bin(
        "update-mirrors",
        ShimCommand::target("bash").paths([update_mirrors_sh.into_utf8()?]),
    ))?;

    // using shell alias for UI-only differences
    let grep_alias = "alias grep='grep --color=auto'";
    ctx.add_item(Item::bash(grep_alias))?;
    ctx.add_item(Item::zsh(grep_alias))?;
    common::ALIAS_VERSION.update()?;

    Ok(())
}
