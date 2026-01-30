//! Additional essential shell utilities

use crate::pre::*;

#[rustfmt::skip]
register_binaries!(
    "perl", "gpg", "curl", "wget",
    "fzf", "jq", "task", "x",
    "bat", "dust", "fd", "websocat", "zoxide", "c", "ci",
    "viopen", "vibash", "vihosts", "n"
);

mod common;
mod perl;

pub fn binary_dependencies() -> EnumSet<BinId> {
    enum_set! { BinId::_7z }
}

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_installed_pacman_package!("perl");
    let v = perl::version_check()?;
    if v != Verified::UpToDate {
        return Ok(v);
    }
    cu::check!(
        cu::which("gpg"),
        "gnupg is a dependency of Arch Linux and is not found"
    )?;
    let v = check_installed_pacman_package!("curl");
    if Version(&v) < metadata::curl::VERSION {
        return Ok(Verified::NotUpToDate);
    }
    let v = check_installed_pacman_package!("wget");
    if Version(&v) < metadata::wget::VERSION {
        return Ok(Verified::NotUpToDate);
    }
    let v = check_installed_pacman_package!("fzf");
    if Version(&v) < metadata::fzf::VERSION {
        return Ok(Verified::NotUpToDate);
    }
    let v = check_installed_pacman_package!("jq");
    if Version(&v) < metadata::jq::VERSION {
        return Ok(Verified::NotUpToDate);
    }
    check_bin_in_path_and_shaft!("task");
    check_bin_in_path_and_shaft!("x");
    let v = command_output!("task", ["--version"]);
    if Version(&v) < metadata::task::VERSION {
        return Ok(Verified::NotUpToDate);
    }
    let v = check_installed_with_cargo!("bat");
    if Version(&v.version) < metadata::bat::VERSION {
        return Ok(Verified::NotUpToDate);
    }
    let v = check_installed_with_cargo!("dust", "du-dust");
    if Version(&v.version) < metadata::dust::VERSION {
        return Ok(Verified::NotUpToDate);
    }
    let v = check_installed_with_cargo!("find", "fd-find");
    if Version(&v.version) < metadata::fd::VERSION {
        return Ok(Verified::NotUpToDate);
    }
    let v = check_installed_with_cargo!("websocat");
    if Version(&v.version) < metadata::websocat::VERSION {
        return Ok(Verified::NotUpToDate);
    }
    let v = check_installed_with_cargo!("zoxide");
    if Version(&v.version) < metadata::zoxide::VERSION {
        return Ok(Verified::NotUpToDate);
    }
    let v = check_installed_with_cargo!("viopen");
    if Version(&v.version) < metadata::shellutils::viopen::VERSION {
        return Ok(Verified::NotUpToDate);
    }
    let v = check_installed_with_cargo!("n");
    if Version(&v.version) < metadata::shellutils::n::VERSION {
        return Ok(Verified::NotUpToDate);
    }
    Ok(Verified::is_uptodate(common::ALIAS_VERSION.is_uptodate()?))
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file("task.tgz", task_url(), metadata::task::SHA, ctx.bar())?;
    Ok(())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    let install_dir = ctx.install_dir();
    cu::fs::make_dir(&install_dir)?;
    let task_tgz = hmgr::paths::download("task.tgz", task_url());
    let temp_dir = hmgr::paths::temp_dir("task-tgz");
    let temp_tgz = temp_dir.join("task.tgz");
    let temp_tar = temp_dir.join("task.tar");
    cu::fs::copy(&task_tgz, &temp_tgz)?;
    opfs::un7z(temp_tgz, &temp_dir, ctx.bar_ref())?;
    opfs::un7z(temp_tar, &temp_dir, ctx.bar_ref())?;
    let task_exe = temp_dir.join(bin_name!("task"));
    cu::fs::copy(task_exe, install_dir.join(bin_name!("task")))?;
    epkg::pacman::install("perl", ctx.bar_ref())?;
    epkg::pacman::install("curl", ctx.bar_ref())?;
    epkg::pacman::install("wget", ctx.bar_ref())?;
    epkg::pacman::install("fzf", ctx.bar_ref())?;
    epkg::pacman::install("jq", ctx.bar_ref())?;
    epkg::cargo::binstall("bat", ctx.bar_ref())?;
    epkg::cargo::binstall("du-dust", ctx.bar_ref())?;
    epkg::cargo::install("fd-find", ctx.bar_ref())?;
    epkg::cargo::install("websocat", ctx.bar_ref())?;
    epkg::cargo::install("zoxide", ctx.bar_ref())?;
    epkg::cargo::install_git_commit(
        "viopen",
        metadata::shellutils::REPO,
        metadata::shellutils::COMMIT,
        ctx.bar_ref(),
    )?;
    epkg::cargo::install_git_commit(
        "n",
        metadata::shellutils::REPO,
        metadata::shellutils::COMMIT,
        ctx.bar_ref(),
    )?;
    Ok(())
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    epkg::pacman::uninstall("perl", ctx.bar_ref())?;
    epkg::pacman::uninstall("curl", ctx.bar_ref())?;
    epkg::pacman::uninstall("wget", ctx.bar_ref())?;
    epkg::pacman::uninstall("fzf", ctx.bar_ref())?;
    epkg::pacman::uninstall("jq", ctx.bar_ref())?;
    epkg::cargo::uninstall("bat")?;
    epkg::cargo::uninstall("du-dust")?;
    epkg::cargo::uninstall("fd-find")?;
    epkg::cargo::uninstall("websocat")?;
    epkg::cargo::uninstall("zoxide")?;
    epkg::cargo::uninstall("viopen")?;
    epkg::cargo::uninstall("n")?;
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let task_exe = ctx.install_dir().join(bin_name!("task")).into_utf8()?;
    ctx.add_item(hmgr::Item::LinkBin(
        hmgr::paths::binary(bin_name!("task")).into_utf8()?,
        task_exe.clone(),
    ))?;
    ctx.add_item(hmgr::Item::LinkBin(
        hmgr::paths::binary(bin_name!("x")).into_utf8()?,
        task_exe.clone(),
    ))?;
    let mut script = command_output!(&task_exe, ["--completion", "bash"]);
    script.push_str("\ncomplete -F _task x");
    ctx.add_item(hmgr::Item::Bash(script))?;
    let mut script = "compdef _task x\n".to_string();
    script.push_str(&command_output!(&task_exe, ["--completion", "zsh"]));
    ctx.add_item(hmgr::Item::Zsh(script))?;

        ctx.add_item(hmgr::Item::UserEnvVar(
            "EDITOR".to_string(),
            "viopen".to_string(),
        ))?;

        // zoxide needs to be after starship, recommended to be at the end
        let script = command_output!("zoxide", ["init", "bash", "--cmd", "c"]);
        ctx.add_priority_item(-1, hmgr::Item::Bash(script))?;
        let script = command_output!("zoxide", ["init", "zsh", "--cmd", "c"]);
        ctx.add_priority_item(-1, hmgr::Item::Zsh(script))?;

        if let Some(mut home) = std::env::home_dir() {
            home.push(".bashrc");
            ctx.add_item(hmgr::Item::ShimBin(
                "vibash".to_string(),
                vec![cu::which("viopen")?.into_utf8()?, home.into_utf8()?],
            ))?;
        }
        ctx.add_item(hmgr::Item::ShimBin(
            "vihosts".to_string(),
            vec![cu::which("viopen")?.into_utf8()?, "/etc/hosts".to_string()],
        ))?;

    common::ALIAS_VERSION.update()?;
    Ok(())
}

fn task_url() -> String {
    let repo = metadata::task::REPO;
    let ver = metadata::task::VERSION;
    format!("{repo}/releases/download/v{ver}/task_linux_amd64.tar.gz")
}
