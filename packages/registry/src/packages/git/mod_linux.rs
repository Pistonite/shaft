//! Git version control System. Uses Microsoft fork on Ubuntu

use crate::pre::*;

mod version;

register_binaries!("git", "scalar");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    if opfs::is_pacman() {
        check_pacman!("git");
        version::verify(metadata::git::PACMAN_VERSION, false)
    } else {
        check_in_path!("git");
        check_in_path!("scalar");
        check_apt!("microsoft-git");
        version::verify(metadata::git::MICROSOFT_VERSION, true)
    }
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    if !opfs::is_pacman() {
        hmgr::download_file(
            deb_filename(),
            metadata::git::URL,
            metadata::git::SHA,
            ctx.bar(),
        )?;
    }
    Ok(())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    opfs::ensure_terminated("git")?;

    if opfs::is_pacman() {
        epkg::pacman::install_many(
            &["pcre2", "git"],
            "install git and dependencies",
            ctx.bar_ref(),
        )?;
    } else {
        let deb = hmgr::paths::download(deb_filename(), metadata::git::URL);
        epkg::apt::install_file(&deb, ctx.bar_ref())?;
    }
    Ok(())
}

fn deb_filename() -> &'static str {
    "microsoft-git.deb"
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    opfs::ensure_terminated("git")?;
    if opfs::is_pacman() {
        epkg::pacman::uninstall("git", ctx.bar_ref())?;
    } else {
        epkg::pacman::uninstall("microsoft-git", ctx.bar_ref())?;
    }
    Ok(())
}
