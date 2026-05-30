//! GNU Coreutils, Diffutils, and other basic commands for common workflows

use std::collections::BTreeSet;

use crate::pre::*;

mod common;
mod eza;
mod which;

register_binaries!("ls", "diff", "find", "gzip", "sed", "grep", "tar");
binary_dependencies!(Git, CargoBinstall);

static PS_ALIASES: &[&str] = &[
    "cat", "cp", "dir", "echo", "ls", "mv", "pwd", "rm", "rmdir", "sort", "sleep", "tee",
];
static SYSTEM32_EXES: &[&str] = &["expand", "hostname", "more", "sort", "whoami"];
static PS_FUNCTIONS: &[&str] = &["mkdir"];

pub fn verify(_: &Context) -> cu::Result<Verified> {
    eza::verify()?;
    check_in_shaft!("diff");
    check_in_shaft!("diff3");
    check_in_shaft!("cmp");
    // not checking find because of System32\find.exe
    check_in_shaft!("gzip");
    check_in_shaft!("sed");
    check_in_shaft!("grep");
    cu::check!(
        cu::which("tar"),
        "tar.exe is bundled in Windows; your Windows version might be too low"
    )?;
    let v = check_cargo!("coreutils");
    check_outdated!(&v.version, metadata[coreutils::uutils]::VERSION);

    check_verified!(which::verify()?);

    check_config_version_cache!(common::ALIAS_VERSION);
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    eza::install(ctx)?;
    epkg::cargo::binstall("coreutils", ctx.bar_ref())?;
    which::install(ctx)?;
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    eza::uninstall()?;
    let coreutils_path = hmgr::paths::binary(bin_name!("coreutils"));
    cu::fs::remove(&coreutils_path)?;
    epkg::cargo::uninstall("coreutils")?;
    which::uninstall(ctx)?;
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    eza::configure(ctx)?;
    // configure coreutils
    // we need to copy installed coreutils to bin to ensure
    // it's on the same drive, so it can be hardlinked
    let old_coreutils_path = hmgr::paths::binary(bin_name!("coreutils"));
    cu::fs::remove(&old_coreutils_path)?;
    let coreutils_path = ctx.install_dir().join(bin_name!("coreutils"));
    let coreutils_src = cu::which("coreutils")?;
    cu::fs::copy(&coreutils_src, &coreutils_path)?;
    let coreutils_path = coreutils_path.into_utf8()?;

    let config = ctx.load_config(CONFIG)?;

    let all_utils = command_output!("coreutils", ["--list"]);
    // ^ shadowed, but need to keep alive
    let all_utils: BTreeSet<_> = all_utils
        .lines()
        .map(|s| s.trim())
        .filter(|s| {
            if s.is_empty() {
                return false;
            }
            if !s.chars().all(|c| c.is_alphanumeric()) {
                return false;
            }
            // excluded by config:
            if config.windows.exclude_coreutils.contains(*s) {
                return false;
            }
            // always-exclude:
            if *s == "link" {
                // link conflicts with MSVC
                return false;
            }

            true
        })
        .collect();

    // link utils
    for util in &all_utils {
        ctx.add_item(Item::link_bin(bin_name!(util), coreutils_path.clone()))?;
    }
    // remove PS aliases and functions
    for util in PS_ALIASES {
        if all_utils.contains(util) {
            ctx.add_item(Item::pwsh(format!("Remove-Item Alias:{util} -Force")))?;
        }
    }
    for util in PS_FUNCTIONS {
        if all_utils.contains(util) {
            ctx.add_item(Item::pwsh(format!("Remove-Item Function:{util} -Force")))?;
        }
    }
    // override System32 binaries by setting alias/doskey
    for util in SYSTEM32_EXES {
        let link_path = hmgr::paths::binary(bin_name!(&util)).into_utf8()?;
        if all_utils.contains(util) {
            ctx.add_item(Item::pwsh(format!(
                "Set-Alias -Name {util} -Value '{link_path}'"
            )))?;
            ctx.add_item(Item::cmd(format!("doskey {util}=\"{link_path}\" $*")))?;
        }
    }
    if config.windows.cmd_mkdir {
        let link_path = hmgr::paths::binary(bin_name!("mkdir")).into_utf8()?;
        ctx.add_item(Item::cmd(format!("doskey mkdir=\"{link_path}\" -p $*")))?;
    }

    // configure utils from mingw
    let exe_path = opfs::find_in_wingit("usr/bin/grep.exe")?;
    ctx.add_item(Item::shim_bin(
        bin_name!("grep"),
        ShimCommand::target(exe_path.into_utf8()?).args(["--color=auto"]),
    ))?;
    const MINGW_UTILS: &[&str] = &["diff", "diff3", "cmp", "find", "gzip", "sed"];
    for util in MINGW_UTILS {
        if config.windows.exclude_coreutils.contains(*util) {
            continue;
        }
        let exe_path = opfs::find_in_wingit(format!("usr/bin/{util}.exe"))?;
        ctx.add_item(Item::shim_bin(
            bin_name!(util),
            ShimCommand::target(exe_path.into_utf8()?),
        ))?;
    }
    if !config.windows.exclude_coreutils.contains("find") {
        let findutil_path = hmgr::paths::binary(bin_name!("find")).into_utf8()?;
        ctx.add_item(Item::pwsh(format!(
            "Set-Alias -Name find -Value '{findutil_path}'"
        )))?;
        ctx.add_item(Item::cmd(format!("doskey find=\"{findutil_path}\" $*")))?;
    }

    common::ALIAS_VERSION.update()?;

    Ok(())
}

config_file! {
    static CONFIG: Config = {
        template: include_str!("config.toml"),
        migration: []
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Config {
    windows: ConfigWindows,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct ConfigWindows {
    exclude_coreutils: BTreeSet<String>,
    cmd_mkdir: bool,
}
