//! Additional essential shell utilities

use crate::pre::*;

#[rustfmt::skip]
register_binaries!(
    "perl", "gpg", "curl", "wget",
    "fzf", "jq", "task", "x",
    "bat", "dust", "fd", "rg", "websocat", "zoxide", "c", "ci",
    "viopen", "vihosts", "n",
    "vipath",
    "wsclip"
);

binary_dependencies!(Scalar, _7z, CargoBinstall);

mod common;
mod wget;

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_in_shaft!("perl");
    check_in_shaft!("gpg");
    cu::check!(
        cu::which("curl"),
        "curl.exe is bundled in Windows; your Windows version might be too low"
    )?;

    check_in_shaft!("wget");
    let v = wget::version_check()?;
    if v != Verified::UpToDate {
        return Ok(v);
    }

    check_in_shaft!("fzf");
    let v = command_output!("fzf", ["--version"]);
    let v = v.split_once(' ').map(|x| x.0).unwrap_or(&v);
    check_outdated!(v, metadata[fzf]::VERSION);

    check_in_shaft!("jq");
    let v = command_output!("jq", ["--version"]);
    let v = v.strip_prefix("jq-").unwrap_or(&v);
    check_outdated!(v, metadata[jq]::VERSION);

    check_in_shaft!("task");
    check_in_shaft!("x");
    let v = command_output!("task", ["--version"]);
    check_outdated!(&v, metadata[task]::VERSION);

    let v = check_cargo!("bat");
    check_outdated!(&v.version, metadata[bat]::VERSION);
    let v = check_cargo!("dust" in crate "du-dust");
    check_outdated!(&v.version, metadata[dust]::VERSION);
    let v = check_cargo!("fd" in crate "fd-find");
    check_outdated!(&v.version, metadata[fd]::VERSION);
    let v = check_cargo!("rg" in crate "ripgrep");
    check_outdated!(&v.version, metadata[rg]::VERSION);
    let v = check_cargo!("websocat");
    check_outdated!(&v.version, metadata[websocat]::VERSION);
    let v = check_cargo!("zoxide");
    check_outdated!(&v.version, metadata[zoxide]::VERSION);
    let v = check_cargo!("viopen");
    check_outdated!(&v.version, metadata[shellutils::viopen]::VERSION);
    let v = check_cargo!("n");
    check_outdated!(&v.version, metadata[shellutils::n]::VERSION);
    let v = check_cargo!("wsclip");
    check_outdated!(&v.version, metadata[shellutils::wsclip]::VERSION);
    let v = check_cargo!("vipath");
    check_outdated!(&v.version, metadata[shellutils::vipath]::VERSION);

    check_version_cache!(common::ALIAS_VERSION);
    Ok(Verified::UpToDate)
}
pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file(
        "wget.7z",
        metadata::wget::URL,
        metadata::wget::SHA,
        ctx.bar(),
    )?;
    hmgr::download_file("fzf.zip", fzf_url(), metadata::fzf::SHA, ctx.bar())?;
    hmgr::download_file("jq.exe", jq_url(), metadata::jq::SHA, ctx.bar())?;
    hmgr::download_file("task.zip", task_url(), metadata::task::SHA, ctx.bar())?;
    Ok(())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    let install_dir = ctx.install_dir();
    cu::fs::make_dir(&install_dir)?;

    let wget_7z = hmgr::paths::download("wget.7z", metadata::wget::URL);
    opfs::unarchive(wget_7z, &install_dir, false)?;

    let fzf_zip = hmgr::paths::download("fzf.zip", fzf_url());
    opfs::unarchive(fzf_zip, &install_dir, false)?;

    let jq_exe = hmgr::paths::download("jq.exe", jq_url());
    let jq_target = install_dir.join(bin_name!("jq"));
    cu::fs::copy(jq_exe, jq_target)?;

    let task_zip = hmgr::paths::download("task.zip", task_url());
    let temp_dir = hmgr::paths::temp_dir("task-zip");
    opfs::unarchive(task_zip, &temp_dir, false)?;
    let task_exe = temp_dir.join(bin_name!("task"));
    cu::fs::copy(task_exe, install_dir.join(bin_name!("task")))?;

    epkg::cargo::binstall("bat", ctx.bar_ref())?;
    epkg::cargo::binstall("du-dust", ctx.bar_ref())?;
    epkg::cargo::install("fd-find", ctx.bar_ref())?;
    epkg::cargo::binstall("ripgrep", ctx.bar_ref())?;
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
    epkg::cargo::install_git_commit(
        "wsclip",
        metadata::shellutils::REPO,
        metadata::shellutils::COMMIT,
        ctx.bar_ref(),
    )?;
    epkg::cargo::install_git_commit(
        "vipath",
        metadata::shellutils::REPO,
        metadata::shellutils::COMMIT,
        ctx.bar_ref(),
    )?;
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    epkg::cargo::uninstall("bat")?;
    epkg::cargo::uninstall("du-dust")?;
    epkg::cargo::uninstall("fd-find")?;
    epkg::cargo::uninstall("websocat")?;
    epkg::cargo::uninstall("zoxide")?;
    epkg::cargo::uninstall("viopen")?;
    epkg::cargo::uninstall("n")?;
    epkg::cargo::uninstall("wsclip")?;
    epkg::cargo::uninstall("vipath")?;
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let exe_path = opfs::find_in_wingit("usr/bin/perl.exe")?;
    ctx.add_item(Item::shim_bin(
        bin_name!("perl"),
        ShimCommand::target(exe_path.into_utf8()?),
    ))?;
    let exe_path = opfs::find_in_wingit("usr/bin/gpg.exe")?;
    ctx.add_item(Item::shim_bin(
        bin_name!("gpg"),
        ShimCommand::target(exe_path.into_utf8()?).bash(),
    ))?;
    ctx.add_item(Item::link_bin(
        hmgr::paths::binary(bin_name!("wget")).into_utf8()?,
        ctx.install_dir().join(bin_name!("wget")).into_utf8()?,
    ))?;
    ctx.add_item(Item::link_bin(
        hmgr::paths::binary(bin_name!("fzf")).into_utf8()?,
        ctx.install_dir().join(bin_name!("fzf")).into_utf8()?,
    ))?;
    ctx.add_item(Item::link_bin(
        hmgr::paths::binary(bin_name!("jq")).into_utf8()?,
        ctx.install_dir().join(bin_name!("jq")).into_utf8()?,
    ))?;
    let task_exe = ctx.install_dir().join(bin_name!("task")).into_utf8()?;
    ctx.add_item(Item::link_bin(
        hmgr::paths::binary(bin_name!("task")).into_utf8()?,
        task_exe.clone(),
    ))?;
    ctx.add_item(Item::link_bin(
        hmgr::paths::binary(bin_name!("x")).into_utf8()?,
        task_exe,
    ))?;
    let script = r#"Invoke-Expression (& {((task --completion powershell).replace("-CommandName task","-CommandName task,x") | Out-String)})"#;
    ctx.add_item(Item::pwsh(script))?;

    ctx.add_item(Item::user_env_var("EDITOR", "viopen"))?;

    // zoxide needs to be after starship, recommended to be at the end
    let script = command_output!("zoxide", ["init", "powershell", "--cmd", "c"]);
    ctx.add_priority_item(-1, Item::pwsh(script))?;
    let install_dir = ctx.install_dir();
    let zoxide_c_cmd = install_dir.join("zoxide_c.cmd");
    let zoxide_ci_cmd = install_dir.join("zoxide_ci.cmd");
    cu::fs::write(&zoxide_c_cmd, include_bytes!("./zoxide_c.cmd"))?;
    cu::fs::write(&zoxide_ci_cmd, include_bytes!("./zoxide_ci.cmd"))?;

    // currently, doing this would allow c/ci to work to jump to directory,
    // but regular cd won't update zoxide database, so it's not really useful
    // ctx.add_item(Item::cmd(format!(
    //     "doskey c=call \"{}\" $1\r\ndoskey ci=call \"{}\" $1",
    //     zoxide_c_cmd.into_utf8()?,
    //     zoxide_ci_cmd.into_utf8()?
    // )))?;

    ctx.add_item(Item::shim_bin(
        "vihosts",
        ShimCommand::target(cu::which("cmd")?.into_utf8()?).args([
            "/c",
            "viopen %SystemDrive%\\Windows\\System32\\drivers\\etc\\hosts",
        ]),
    ))?;

    common::ALIAS_VERSION.update()?;
    Ok(())
}

fn fzf_url() -> String {
    let repo = metadata::fzf::REPO;
    let ver = metadata::fzf::VERSION;
    let arch = if_arm!("arm64", else "amd64");
    format!("{repo}/releases/download/v{ver}/fzf-{ver}-windows_{arch}.zip")
}

fn jq_url() -> String {
    let repo = metadata::jq::REPO;
    let ver = metadata::jq::VERSION;
    format!("{repo}/releases/download/jq-{ver}/jq-windows-amd64.exe")
}

fn task_url() -> String {
    let arch = if_arm!("arm64", else "amd64");
    let repo = metadata::task::REPO;
    let ver = metadata::task::VERSION;
    format!("{repo}/releases/download/v{ver}/task_windows_{arch}.zip")
}
