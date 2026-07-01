//! Additional essential shell utilities

use crate::pre::*;

#[rustfmt::skip]
register_binaries!(
    "perl", "gpg", "curl", "wget",
    "fzf", "jq", "task", "x",
    "bat", "dust", "fd", "rg", "websocat", "zoxide", "c", "ci",
    "viopen", "vihosts", "n", "lfmt",
    "vipath",
    "wsclip"
);

binary_dependencies!(Scalar, _7z, CargoBinstall);

mod cargoones;
mod common;
mod shutil;
mod task;
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
    check_outdated!(v.trim(), metadata[fzf]::VERSION);

    check_in_shaft!("jq");
    let v = command_output!("jq", ["--version"]);
    let v = v.strip_prefix("jq-").unwrap_or(&v);
    check_outdated!(v.trim(), metadata[jq]::VERSION);

    check_verified!(task::verify()?);
    check_verified!(cargoones::verify()?);
    check_verified!(shutil::verify()?);

    check_config_version_cache!(common::ALIAS_VERSION);
    Ok(Verified::UpToDate)
}
pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file(
        "wget.7z",
        metadata::wget::URL,
        metadata::wget::SHA,
        ctx.bar(),
    )?;
    hmgr::download_file("fzf.zip", fzf_url(), metadata::fzf::SHA(), ctx.bar())?;
    hmgr::download_file("jq.exe", jq_url(), metadata::jq::SHA, ctx.bar())?;
    task::download(ctx)?;
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

    task::install(ctx)?;
    cargoones::install(ctx)?;
    shutil::install(ctx)?;
    Ok(())
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    cargoones::uninstall(ctx)?;
    shutil::uninstall(ctx)?;
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

    task::configure(ctx)?;

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
        bin_name!("vihosts"),
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
    let arch = if opfs::is_arm() { "arm64" } else { "amd64" };
    format!("{repo}/releases/download/v{ver}/fzf-{ver}-windows_{arch}.zip")
}

fn jq_url() -> String {
    let repo = metadata::jq::REPO;
    let ver = metadata::jq::VERSION;
    format!("{repo}/releases/download/jq-{ver}/jq-windows-amd64.exe")
}
