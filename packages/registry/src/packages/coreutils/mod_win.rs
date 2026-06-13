//! GNU Coreutils, Diffutils, and other basic commands for common workflows

use crate::pre::*;

mod common;
mod eza;
mod sed;
mod which;

register_binaries!(
    "tar", // builtin
    "ls",  // eza
    // uutils/sed
    "sed",
    // ms-coreutils
    "coreutils",
    "find",
    "sort",
    "grep",
    "xargs",
    // mingw
    "diff",
    "gzip"
);
binary_dependencies!(Git, CargoBinstall);

// static PS_ALIASES: &[&str] = &[
//     "cat", "cp", "dir", "echo", "ls", "mv", "pwd", "rm", "rmdir", "sort", "sleep", "tee",
// ];
// keep in sync with pwsh-install-template.ps1
#[rustfmt::skip]
static MS_COREUTILS_LIST: &[&str] = &[
    "arch", "b2sum", "base32", "base64", "basename",
    "basenc", "cat", "cksum", "comm", "cp",
    "csplit", "cut", "df", "dirname",
    "du", "echo", "env", "expr", "factor",
    "false", "find", "fmt", "fold", "grep",
    "head", "hostname", "join", "la",
    "ln", "md5sum", "mkdir", "mktemp",
    "mv", "nl", "nproc", "numfmt", "od",
    "pathchk", "pr", "printenv", "printf", "ptx",
    "pwd", "readlink", "realpath", "rm",
    "seq", "sha1sum", "sha224sum", "sha256sum", "sha384sum",
    "sha512sum", "shuf", "sleep", "sort", "split",
    "stat", "sum", "tac", "tail", "tee",
    "test", "touch", "tr", "true", "truncate",
    "tsort", "unexpand", "uniq", "unlink", "uptime",
    "wc", "xargs", "yes"
];
// these are compatible with DOS and will be put in sbin/ to superceed System32/ ones
static MS_COREUTILS_SBIN_LIST: &[&str] = &["find", "sort", "hostname"];
// static PS_REMOVE_SYSTEM32_EXES: &[&str] = &["expand", "hostname", "sort", ];

// static PS_FUNCTIONS: &[&str] = &["mkdir"];

version_cache!(static MS_COREUTILS_COMMIT = metadata::coreutils::ms_coreutils::COMMIT);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_sbin_path()?;

    cu::check!(
        cu::which("tar"),
        "tar.exe is bundled in Windows; your Windows version might be too low"
    )?;
    check_verified!(which::verify()?);

    check_verified!(eza::verify()?);
    check_verified!(sed::verify()?);

    // ms-coreutils: uutils/coreutils + findutils + grep
    check_verified!(verify_coreutils_version()?);
    check_in_shaft!(
        #[sbin]
        "find"
    );
    check_in_shaft!(
        #[sbin]
        "sort"
    );
    check_in_shaft!("grep");

    // mingw
    check_in_shaft!("diff");
    check_in_shaft!("diff3");
    check_in_shaft!("cmp");
    check_in_shaft!("gzip");

    check_config_version_cache!(common::ALIAS_VERSION);
    Ok(Verified::UpToDate)
}

fn check_sbin_path() -> cu::Result<()> {
    // this is best effort check and does not cover every whacky
    // way your PATH is messed up
    let system_paths = hmgr::windows::get_system("PATH")?;
    let system_path_expected = {
        let mut system_root = cu::env_var("SystemRoot")?;
        system_root.make_ascii_lowercase();
        if !system_root.ends_with('\\') {
            system_root.push('\\');
        }
        system_root.push_str("system32");
        system_root
    };
    let mut sbin_path = hmgr::paths::sbin_root().into_utf8()?;
    sbin_path.make_ascii_lowercase();
    for p in system_paths.split(';') {
        let mut p = p.to_lowercase();
        while p.ends_with('\\') || p.ends_with('/') {
            p.pop();
        }
        cu::debug!("{p}");
        if p == sbin_path {
            // sbin path verified to exist and be before system_root/system32
            return Ok(());
        }
        if p == system_path_expected
            || p == "%systemroot%\\system32"
            || p == "%systemdrive%\\windows\\system32"
        {
            break;
        }
    }
    cu::bail!(
        "path check failed: $SHAFT_HOME\\sbin should appear before {system_path_expected} for coreutils to work. Please fix the SYSTEM PATH"
    );
}

fn verify_coreutils_version() -> cu::Result<Verified> {
    check_version_cache!(MS_COREUTILS_COMMIT);
    check_in_shaft!("coreutils");
    let v = get_coreutils_version()?;
    check_outdated!(&v, metadata[coreutils::ms_coreutils]::VERSION);
    Ok(Verified::UpToDate)
}

fn get_coreutils_version() -> cu::Result<String> {
    let output = command_output!("coreutils", ["--version"]);
    let output = output.strip_prefix("coreutils ").unwrap_or(&output);
    let output = output.split_once(' ').map(|a| a.0).unwrap_or(output);
    Ok(output.trim().to_string())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    which::install(ctx)?;
    eza::install(ctx)?;
    sed::install(ctx)?;
    let coreutils_dir = cu::path!((ctx.install_dir()) / "coreutils");
    let coreutils_bin = cu::path!(&coreutils_dir / "bin" / bin_name!("coreutils"));
    if coreutils_bin.exists()
        && let Ok(Verified::UpToDate) = verify_coreutils_version()
    {
        return Ok(());
    }

    epkg::cargo::install_git_commit(
        "coreutils",
        metadata::coreutils::ms_coreutils::REPO,
        metadata::coreutils::ms_coreutils::COMMIT,
        Some(coreutils_dir.as_utf8()?),
        ctx.bar_ref(),
    )?;

    MS_COREUTILS_COMMIT.update()?;
    Ok(())
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    which::uninstall(ctx)?;
    sed::uninstall(ctx)?;
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    eza::configure(ctx)?;
    // configure coreutils
    // see https://github.com/microsoft/coreutils/blob/main/coreutils.iss
    let config = ctx.load_config(CONFIG)?;

    let curr_prefer_gnu_find = get_ms_coreutils_reg_value("DefaultFind").unwrap_or_default();
    let curr_prefer_gnu_find = curr_prefer_gnu_find.trim() == "1";
    let curr_prefer_gnu_sort = get_ms_coreutils_reg_value("DefaultSort").unwrap_or_default();
    let curr_prefer_gnu_sort = curr_prefer_gnu_sort.trim() == "1";

    let need_set_reg = curr_prefer_gnu_find != config.windows.prefer_gnu
        || curr_prefer_gnu_sort != config.windows.prefer_gnu;
    if need_set_reg {
        cu::debug!("setting registry for ms-coreutils");
        let script = format!(
            concat!(
                r"New-Item -Path 'HKLM:\SOFTWARE\Microsoft\coreutils' -Force | Out-Null;",
                r"New-ItemProperty -Path 'HKLM:\SOFTWARE\Microsoft\coreutils' -Name 'DefaultFind' -PropertyType DWord -Value {0} -Force | Out-Null;",
                r"New-ItemProperty -Path 'HKLM:\SOFTWARE\Microsoft\coreutils' -Name 'DefaultSort' -PropertyType DWord -Value {0} -Force | Out-Null;",
            ),
            if config.windows.prefer_gnu { "1" } else { "0" }
        );

        opfs::sudo("powershell", "setting registry values for ms-coreutils")?
            .args(["-NoLogo", "-NoProfile", "-Command", &script])
            .stdoe(cu::lv::D)
            .stdin_null()
            .wait_nz()?;
    }

    let coreutils_dir = cu::path!((ctx.install_dir()) / "coreutils");
    let coreutils_bin = cu::path!(&coreutils_dir / "bin" / bin_name!("coreutils")).into_utf8()?;
    // make the .cmd links - see pwsh-install-template.ps1 why
    let coreutils_cmd_dir = {
        let mut p = coreutils_dir.join("cmd").into_utf8()?;
        if !p.ends_with('\\') {
            p.push('\\')
        }
        p
    };
    cu::fs::make_dir_empty(&coreutils_cmd_dir)?;
    let mut link_paths = Vec::with_capacity(MS_COREUTILS_LIST.len());
    for util in MS_COREUTILS_LIST {
        let link_target = format!("{coreutils_cmd_dir}\\{util}.cmd");
        link_paths.push((link_target, &coreutils_bin));
    }
    cu::check!(
        opfs::hardlink_files(link_paths.into_iter()),
        "failed to create cmd links for ms-coreutils"
    )?;
    // link bins
    ctx.add_item(Item::link_bin(
        bin_name!("coreutils"),
        coreutils_bin.clone(),
    ))?;
    for util in MS_COREUTILS_LIST {
        ctx.add_item(Item::link_bin(bin_name!(util), coreutils_bin.clone()))?;
    }
    for util in MS_COREUTILS_SBIN_LIST {
        ctx.add_item(Item::link_sys_bin(bin_name!(util), coreutils_bin.clone()))?;
    }
    // add the pwsh injection script
    // need to refer to https://github.com/microsoft/coreutils/blob/main/src/pwsh-install.ps1
    // if changes in the future
    let pwsh_injection_script =
        include_str!("pwsh-install-template.ps1").replace("!!CMDDIR!!", &coreutils_cmd_dir);

    ctx.add_item(Item::pwsh(format!(
        r###"
if ($PSVersionTable.PSVersion.Major -ne 5) {{
#--- SHAFT 
{ps7}
#--- SHAFT
}} else {{
#--- SHAFT
{ps5}
#--- SHAFT
}}
"###,
        ps7 = pwsh_injection_script,
        ps5 = include_str!("ps5-compat.ps1")
    )))?;
    if config.windows.cmd_mkdir {
        let link_path = hmgr::paths::binary(bin_name!("mkdir")).into_utf8()?;
        ctx.add_item(Item::cmd(format!("doskey mkdir=\"{link_path}\" -p $*")))?;
    }
    for util in MS_COREUTILS_SBIN_LIST {
        let util_path = hmgr::paths::binary(bin_name!(util)).into_utf8()?;
        ctx.add_item(Item::cmd(format!("doskey {util_path}=\"{util_path}\" $*")))?;
    }

    const MINGW_UTILS: &[&str] = &["diff", "diff3", "cmp", "gzip"];
    for util in MINGW_UTILS {
        let exe_path = opfs::find_in_wingit(format!("usr/bin/{util}.exe"))?;
        ctx.add_item(Item::shim_bin(
            bin_name!(util),
            ShimCommand::target(exe_path.into_utf8()?),
        ))?;
    }

    common::ALIAS_VERSION.update()?;

    Ok(())
}

fn get_ms_coreutils_reg_value(value: &str) -> cu::Result<String> {
    let script =
        format!("(Get-ItemProperty -Path 'HKLM:\\SOFTWARE\\Microsoft\\coreutils').{value}");
    let (child, output) = cu::which("powershell")?
        .command()
        .args(["-NoLogo", "-NoProfile", "-Command", &script])
        .stdout(cu::pio::string())
        .stderr(cu::lv::D)
        .stdin_null()
        .spawn()?;
    child.wait_nz()?;
    let output = output.join()??;
    Ok(output)
}

config_file! {
    static CONFIG: Config = {
        template: include_str!("config.toml"),
        migration: [""]
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
    prefer_gnu: bool,
    cmd_mkdir: bool,
}
