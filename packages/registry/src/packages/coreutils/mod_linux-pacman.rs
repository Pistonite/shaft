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
    "pacman-update"
);
binary_dependencies!(Git);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    eza::verify()?;
    check_in_shaft!("pacman-update");
    check_pacman!("base");
    check_pacman!("reflector");

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

    let v = check_pacman!("yay-bin");
    check_outdated!(&v, metadata[coreutils::yay]::VERSION);

    check_version_cache!(common::ALIAS_VERSION);
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    eza::install(ctx)?;
    let install_dir = ctx.install_dir();
    let update_sh = install_dir.join("pacman-update.sh");
    cu::fs::write(update_sh, include_bytes!("./pacman-update.sh"))?;

    // manually install yay-bin
    {
        let bar = cu::progress("installing yay-bin").parent(ctx.bar()).spawn();
        let yay_dir = install_dir.join("yay-bin");
        cu::fs::make_dir_absent_or_empty(&yay_dir)?;
        cu::which("git")?
            .command()
            .add(cu::args![
                "-C",
                &install_dir,
                "clone",
                "https://aur.archlinux.org/yay-bin.git"
            ])
            .stdoe(cu::lv::D)
            .stdin_null()
            .wait_nz()?;
        cu::which("makepkg")?
            .command()
            .current_dir(&yay_dir)
            .stdoe(cu::lv::D)
            .stdin_null()
            .wait_nz()?;
        let pkg_file = cu::fs::read_dir(&yay_dir)?
            .filter_map(|entry| {
                let Ok(entry) = entry else {
                    return None;
                };
                let Ok(file_name) = entry.file_name().into_utf8() else {
                    return None;
                };
                if !file_name.ends_with(".pkg.tar.zst") {
                    return None;
                }
                if file_name.contains("debug") {
                    return None;
                }
                Some(entry.path())
            })
            .next();
        let pkg_file = cu::check!(pkg_file, "failed to find pkg file in yay-bin after makepkg")?;
        epkg::pacman::install_file(&pkg_file, Some(&bar))?;
        bar.done();
    }

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
    let update_sh = install_dir.join("pacman-update.sh");

    ctx.add_item(Item::link_bin("pacman-update", update_sh.into_utf8()?))?;

    // using shell alias for UI-only differences
    let grep_alias = "alias grep='grep --color=auto'";
    ctx.add_item(Item::bash(grep_alias))?;
    ctx.add_item(Item::zsh(grep_alias))?;
    common::ALIAS_VERSION.update()?;

    Ok(())
}
