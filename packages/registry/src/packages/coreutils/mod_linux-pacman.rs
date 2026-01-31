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
    check_bin_in_path_and_shaft!("update-mirrors");
    check_installed_pacman_package!("base");
    let v = check_installed_pacman_package!("bash");
    if Version(&v) < metadata::coreutils::bash::VERSION {
        return Ok(Verified::NotUpToDate);
    }
    let v = check_installed_pacman_package!("bash-completion");
    if Version(&v) < metadata::coreutils::bash_cmp::VERSION {
        return Ok(Verified::NotUpToDate);
    }

    let v = check_installed_pacman_package!("zip");
    if Version(&v) < metadata::coreutils::zip::VERSION {
        return Ok(Verified::NotUpToDate);
    }
    let v = check_installed_pacman_package!("unzip");
    if Version(&v) < metadata::coreutils::unzip::VERSION {
        return Ok(Verified::NotUpToDate);
    }
    let v = check_installed_pacman_package!("tar");
    if Version(&v) < metadata::coreutils::tar::VERSION {
        return Ok(Verified::NotUpToDate);
    }
    let v = check_installed_pacman_package!("which");
    if Version(&v) < metadata::coreutils::which::VERSION {
        return Ok(Verified::NotUpToDate);
    }
    Ok(Verified::is_uptodate(common::ALIAS_VERSION.is_uptodate()?))
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

    ctx.add_item(hmgr::Item::ShimBin(
        "update-mirrors".to_string(),
        vec![
            cu::which("bash")?.into_utf8()?,
            update_mirrors_sh.into_utf8()?,
        ],
    ))?;

    // using shell alias for UI-only differences
    let grep_alias = "alias grep='grep --color=auto'";
    ctx.add_item(hmgr::Item::Bash(grep_alias.to_string()))?;
    ctx.add_item(hmgr::Item::Zsh(grep_alias.to_string()))?;
    common::ALIAS_VERSION.update()?;

    Ok(())
}
