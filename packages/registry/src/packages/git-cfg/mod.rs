//! Configuration for Git

use crate::pre::*;

register_binaries!("delta");
version_cache!(pub static VERSION = metadata::git::CFG_VERSION);
binary_dependencies!(Git);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    let v = check_cargo!("delta" in crate "git-delta");
    check_outdated!(&v.version, metadata[git::delta]::VERSION);

    if opfs::is_wsl() {
        // make sure git-credential-manager.exe is reachable
        cu::check!(
            get_wsl_git_credential_manager_path(),
            "git-credential-manager.exe must be accessible"
        )?;
    }

    check_config_version_cache!(VERSION);
    Ok(Verified::UpToDate)
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::cargo::install("git-delta", None, ctx.bar_ref())?;
    Ok(())
}
pub fn uninstall(_: &Context) -> cu::Result<()> {
    epkg::cargo::uninstall("git-delta")?;
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let config = ctx.load_config(CONFIG)?;
    let cfg_autocrlf = if cfg!(windows) {
        config.autocrlf
    } else {
        false
    };
    if config.editor {
        command_output!("git", ["config", "--global", "core.editor", "viopen"]);
    } else {
        command_output!("git", ["config", "unset", "--global", "core.editor"]);
    }
    command_output!(
        "git",
        [
            "config",
            "--global",
            "core.autocrlf",
            &cfg_autocrlf.to_string()
        ]
    );
    if config.delta {
        command_output!("git", ["config", "--global", "core.pager", "delta"]);
        command_output!(
            "git",
            [
                "config",
                "--global",
                "interactive.diffFilter",
                "delta --color-only"
            ]
        );
        command_output!("git", ["config", "--global", "delta.navigate", "true"]);
        command_output!("git", ["config", "--global", "delta.side-by-side", "true"]);
        command_output!("git", ["config", "--global", "delta.line-numbers", "true"]);
        command_output!(
            "git",
            ["config", "--global", "merge.conflictStyle", "zdiff3"]
        );
    } else {
        command_output!("git", ["config", "unset", "--global", "core.pager"]);
        command_output!(
            "git",
            ["config", "unset", "--global", "interactive.diffFilter"]
        );
        command_output!("git", ["config", "unset", "--global", "delta.navigate"]);
        command_output!("git", ["config", "unset", "--global", "delta.side-by-side"]);
        command_output!("git", ["config", "unset", "--global", "delta.line-numbers"]);
        command_output!(
            "git",
            ["config", "unset", "--global", "merge.conflictStyle"]
        );
    }
    // other configs
    command_output!("git", ["config", "--global", "init.defaultBranch", "main"]);

    // wsl configs
    if opfs::is_wsl() {
        let gcm_exe = get_wsl_git_credential_manager_path()?;
        command_output!("git", ["config", "--global", "credential.helper", gcm_exe]);
        // azure issue with url paths
        command_output!(
            "git",
            [
                "config",
                "--global",
                "credential.https://dev.azure.com.useHttpPath",
                "true"
            ]
        );
    }

    VERSION.update()?;
    Ok(())
}

fn get_wsl_git_credential_manager_path() -> cu::Result<PathBuf> {
    let system_drive = opfs::system_drive()?;
    let system_drive = system_drive.trim_end_matches([' ', ':']);
    let system_drive = system_drive.to_lowercase();
    let git_path = PathBuf::from(format!("/mnt/{system_drive}/Program Files/Git"));
    if !git_path.is_dir() {
        cu::bail!("failed to get git-credential-manager: Git is not installed on Windows!");
    }
    let mut plat_path = None;
    for plat in ["mingw64", "clangarm64"] {
        let p = git_path.join(plat);
        if !p.is_dir() {
            continue;
        }
        plat_path = Some(p);
        break;
    }

    let plat_path = cu::check!(
        plat_path,
        "failed to get git-credential-manager: cannot find architecture/platform directory in Git installation"
    )?;
    Ok(cu::path!(plat_path / "bin" / "git-credential-manager.exe"))
}

config_file! {
    static CONFIG: Config = {
        template: include_str!("config.toml"),
        migration: [],
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Config {
    #[serde(default = "default_true")]
    pub editor: bool,
    #[serde(default = "default_true")]
    pub autocrlf: bool,
    #[serde(default = "default_true")]
    pub delta: bool,
}
fn default_true() -> bool {
    true
}
