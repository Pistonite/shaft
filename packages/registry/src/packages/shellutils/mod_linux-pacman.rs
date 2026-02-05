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

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_pacman!("perl");
    check_verified!(perl::version_check()?);
    cu::check!(
        cu::which("gpg"),
        "gnupg is a dependency of Arch Linux and is not found"
    )?;

    let v = check_pacman!("curl");
    check_outdated!(&v, metadata[curl]::VERSION);
    let v = check_pacman!("wget");
    check_outdated!(&v, metadata[wget]::VERSION);
    let v = check_pacman!("fzf");
    check_outdated!(&v, metadata[fzf]::VERSION);
    let v = check_pacman!("jq");
    check_outdated!(&v, metadata[jq]::VERSION);

    check_in_shaft!("task");
    check_in_shaft!("x");
    let v = command_output!("task", ["--version"]);
    check_outdated!(&v, metadata[task]::VERSION);

    let v = check_cargo!("bat");
    check_outdated!(&v.version, metadata[bat]::VERSION);
    let v = check_cargo!("dust" in crate "du-dust");
    check_outdated!(&v.version, metadata[dust]::VERSION);
    let v = check_cargo!("find" in crate "fd-find");
    check_outdated!(&v.version, metadata[fd]::VERSION);
    let v = check_cargo!("websocat");
    check_outdated!(&v.version, metadata[websocat]::VERSION);
    let v = check_cargo!("zoxide");
    check_outdated!(&v.version, metadata[zoxide]::VERSION);
    let v = check_cargo!("viopen");
    check_outdated!(&v.version, metadata[shellutils::viopen]::VERSION);
    let v = check_cargo!("n");
    check_outdated!(&v.version, metadata[shellutils::n]::VERSION);

    check_version_cache!(common::ALIAS_VERSION);
    Ok(Verified::UpToDate)
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file("task.tgz", task_url(), metadata::task::SHA, ctx.bar())?;
    Ok(())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    let install_dir = ctx.install_dir();
    cu::fs::make_dir(&install_dir)?;

    let task_tgz = hmgr::paths::download("task.tgz", task_url());
    let task_temp = hmgr::paths::temp_dir("task-unarchive");
    opfs::unarchive(task_tgz, &task_temp, true)?;
    cu::fs::copy(task_temp.join("task"), install_dir.join("task"))?;

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
    ctx.add_item(Item::link_bin(
        hmgr::paths::binary(bin_name!("task")).into_utf8()?,
        task_exe.clone(),
    ))?;
    ctx.add_item(Item::link_bin(
        hmgr::paths::binary(bin_name!("x")).into_utf8()?,
        task_exe.clone(),
    ))?;
    let mut script = command_output!(&task_exe, ["--completion", "bash"]);
    script.push_str("\ncomplete -F _task x");
    ctx.add_item(Item::bash(script))?;
    let mut script = "compdef _task x\n".to_string();
    script.push_str(&command_output!(&task_exe, ["--completion", "zsh"]));
    ctx.add_item(Item::zsh(script))?;

    ctx.add_item(Item::user_env_var("EDITOR", "viopen"))?;

    // zoxide needs to be after starship, recommended to be at the end
    let script = command_output!("zoxide", ["init", "bash", "--cmd", "c"]);
    ctx.add_priority_item(-1, Item::bash(script))?;
    let script = command_output!("zoxide", ["init", "zsh", "--cmd", "c"]);
    ctx.add_priority_item(-1, Item::zsh(script))?;

    if let Some(mut home) = std::env::home_dir() {
        home.push(".bashrc");
        ctx.add_item(Item::shim_bin(
            "vibash",
            ShimCommand::target("viopen").args([home.into_utf8()?]),
        ))?;
    }
    ctx.add_item(Item::shim_bin(
        "vihosts",
        ShimCommand::target("viopen").args(["/etc/hosts"]),
    ))?;

    common::ALIAS_VERSION.update()?;
    Ok(())
}

fn task_url() -> String {
    let repo = metadata::task::REPO;
    let ver = metadata::task::VERSION;
    format!("{repo}/releases/download/v{ver}/task_linux_amd64.tar.gz")
}
