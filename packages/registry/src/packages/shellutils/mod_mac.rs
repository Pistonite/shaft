//! Additional essential shell utilities

use crate::pre::*;

#[rustfmt::skip]
register_binaries!(
    "perl", "curl", "wget",
    "fzf", "jq", "task", "x",
    "bat", "dust", "fd", "rg", "websocat", "zoxide", "c", "ci",
    "viopen", "vizsh", "vihosts", "n", "lfmt"
);
binary_dependencies!(CargoBinstall);

mod cargoones;
mod common;
mod shutil;
mod task;

pub fn verify(_: &Context) -> cu::Result<Verified> {
    // perl - use system
    check_in_path!("perl");
    // gpg - todo: don't need it right now
    // curl - use system
    check_in_path!("curl");

    let v = check_homebrew!("wget");
    check_outdated!(&v, metadata[wget]::VERSION);
    let v = check_homebrew!("fzf");
    check_outdated!(&v, metadata[fzf]::VERSION);
    let v = check_homebrew!("jq");
    check_outdated!(&v, metadata[jq]::VERSION);

    check_verified!(task::verify()?);
    check_verified!(cargoones::verify()?);
    check_verified!(shutil::verify()?);

    check_config_version_cache!(common::ALIAS_VERSION);
    Ok(Verified::UpToDate)
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    task::download(ctx)?;
    Ok(())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::brew::install("wget", ctx.bar_ref())?;
    epkg::brew::install("fzf", ctx.bar_ref())?;
    epkg::brew::install("jq", ctx.bar_ref())?;
    task::install(ctx)?;
    cargoones::install(ctx)?;
    shutil::install(ctx)?;
    Ok(())
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    epkg::brew::uninstall("wget", ctx.bar_ref())?;
    epkg::brew::uninstall("fzf", ctx.bar_ref())?;
    epkg::brew::uninstall("jq", ctx.bar_ref())?;
    cargoones::uninstall(ctx)?;
    shutil::uninstall(ctx)?;
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    task::configure(ctx)?;

    ctx.add_item(Item::user_env_var("EDITOR", "viopen"))?;

    // zoxide needs to be after starship, recommended to be at the end
    let script = command_output!("zoxide", ["init", "bash", "--cmd", "c"]);
    ctx.add_priority_item(-1, Item::bash(script))?;
    let script = command_output!("zoxide", ["init", "zsh", "--cmd", "c"]);
    ctx.add_priority_item(-1, Item::zsh(script))?;

    if let Some(mut home) = std::env::home_dir() {
        home.push(".zshrc");
        ctx.add_item(Item::shim_bin(
            "vizsh",
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
