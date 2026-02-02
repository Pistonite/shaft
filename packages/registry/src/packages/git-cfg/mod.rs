//! Configuration for Git

use crate::pre::*;

register_binaries!("delta");

pub static VERSION: VersionCache = VersionCache::new("git-cfg", metadata::git::CFG_VERSION);

pub fn binary_dependencies() -> EnumSet<BinId> {
    enum_set! { BinId::Git }
}

pub fn verify(_: &Context) -> cu::Result<Verified> {
    let v = check_installed_with_cargo!("delta", "git-delta");
    check_outdated!(&v.version, metadata::git::delta::VERSION);
    Ok(Verified::is_uptodate(VERSION.is_uptodate()?))
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::cargo::binstall("git-delta", ctx.bar_ref())?;
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
    VERSION.update()?;
    Ok(())
}

pub fn config_location(ctx: &Context) -> cu::Result<Option<PathBuf>> {
    Ok(Some(ctx.config_file()))
}
static CONFIG: ConfigDef<Config> = ConfigDef::new(include_str!("config.toml"), &[]);
test_config!(CONFIG);
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
