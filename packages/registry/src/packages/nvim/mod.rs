//! Neovim (with configuration)
use enumset::EnumSet;

use crate::pre::*;

register_binaries!("tree-sitter", "vi", "vim", "nvim");
binary_dependencies!(
    Git,      // various
    Clang,    // compile tree sitter
    Diff,     // undo tree
    Websocat, // yank to host
    Fzf,      // various
    Rg,       // various
    Python,   // setup, various
    Node      // installing lsp
);
config_dependencies!(Shellutils); // vinvim

version_cache!(static CFG = metadata::nvim::NVIM_CFG);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    let v = check_cargo!("tree-sitter" in crate "tree-sitter-cli");
    check_outdated!(&v.version, metadata[nvim::treesitter_cli]::VERSION);
    check_in_shaft!("nvim");
    let stdout = command_output!("nvim", ["--version"]);
    let version_line = stdout.lines().next().unwrap_or("");
    let Some(version) = version_line.strip_prefix("NVIM v") else {
        cu::warn!("nvim --version returned unexpected output: {stdout}");
        return Ok(Verified::NotUpToDate);
    };
    check_outdated!(version, metadata[nvim]::VERSION);
    check_version_cache!(CFG);
    Ok(Verified::UpToDate)
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    let file_name = nvim_file_name()?;
    hmgr::download_file(file_name, nvim_url()?, metadata::nvim::SHA, ctx.bar())?;
    Ok(())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::cargo::install("tree-sitter-cli", ctx.bar_ref())?;

    opfs::ensure_terminated(bin_name!("nvim"))?;
    let file_name = nvim_file_name()?;
    let file_stem = cu::check!(
        Path::new(file_name).file_stem(),
        "failed to get nvim file stem"
    )?;
    let file_stem = Path::new(file_stem).file_stem().unwrap_or(file_stem); // for .tar.gz
    let archive_path = hmgr::paths::download(file_name, nvim_url()?);
    let temp_dir = hmgr::paths::temp_dir("nvim-extract");
    opfs::unarchive(archive_path, &temp_dir, true)?;
    ctx.move_install_to_old_if_exists()?;
    let install_dir = ctx.install_dir();
    cu::fs::rename(temp_dir.join(file_stem), install_dir)?;
    Ok(())
}

fn nvim_url() -> cu::Result<String> {
    let repo = metadata::nvim::REPO;
    let version = metadata::nvim::VERSION;
    let file_name = nvim_file_name()?;
    Ok(format!("{repo}/releases/download/v{version}/{file_name}"))
}

fn nvim_file_name() -> cu::Result<&'static str> {
    if cfg!(windows) {
        Ok(if_arm!("nvim-win-arm64.zip", else "nvim-win64.zip"))
    } else if cfg!(target_os = "linux") {
        Ok("nvim-linux-x86_64.tar.gz")
    } else {
        cu::bail!("macos unsupported yet")
    }
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let install_bin = {
        let mut p = ctx.install_dir();
        p.extend(["bin", bin_name!("nvim")]);
        p
    };
    let install_bin_str = install_bin.as_utf8()?;
    ctx.add_item(Item::shim_bin(
        bin_name!("nvim"),
        ShimCommand::target(install_bin_str),
    ))?;
    ctx.add_item(Item::shim_bin(
        bin_name!("vim"),
        ShimCommand::target(install_bin_str),
    ))?;
    ctx.add_item(Item::shim_bin(
        bin_name!("vi"),
        ShimCommand::target(install_bin_str),
    ))?;

    let config_dir = nvim_config_dir()?;

    if ctx.is_installed(PkgId::Shellutils) {
        ctx.add_item(Item::shim_bin(
            bin_name!("vinvim"),
            // we use nvim to open nvim config, if nvim is not installed on the system
            // then there's no point edit the config anyway
            ShimCommand::target(install_bin_str).args([config_dir.as_utf8()?]),
        ))?;
    }

    // copy config files

    hmgr::tools::ensure_unpacked()?;
    let config = ctx.load_config(CONFIG)?;
    let bundled_config_dir = {
        let mut p = hmgr::paths::tools_root();
        p.extend(["nvim", "config"]);
        p
    };
    // copy root files over
    for entry in cu::fs::read_dir(&bundled_config_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name();
        cu::fs::copy(bundled_config_dir.join(&name), config_dir.join(&name))?;
    }
    // copy lua files
    cu::fs::rec_remove(config_dir.join("lua"))?;
    cu::fs::rec_copy_inefficiently(bundled_config_dir.join("lua"), config_dir.join("lua"))?;
    // write config
    let config_lua = config.to_lua();
    let config_gen_path = {
        let mut p = config_dir.clone();
        p.extend(["lua", "piston", "config_gen.lua"]);
        p
    };
    cu::fs::write(config_gen_path, config_lua)?;
    // invoke config
    cu::which("python")?
        .command()
        .args(["setup.py", "apply"])
        .current_dir(&config_dir)
        .stdoe(cu::lv::P)
        .stdin_null()
        .wait_nz()?;
    // delete old lock file
    cu::fs::remove(config_dir.join("lazy-lock.json"))?;
    cu::hint!("run `vi`, `:Lazy` and press `U` to update the plugins");

    CFG.update()?;
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    epkg::cargo::uninstall("tree-sitter-cli")?;
    Ok(())
}

pub fn clean(_: &Context) -> cu::Result<()> {
    cu::which("python")?
        .command()
        .args(["setup.py", "clean"])
        .current_dir(nvim_config_dir()?)
        .stdoe(cu::lv::P)
        .stdin_null()
        .wait_nz()?;
    Ok(())
}

fn nvim_config_dir() -> cu::Result<PathBuf> {
    if cfg!(windows) {
        let mut p = PathBuf::from(cu::env_var("LOCALAPPDATA")?);
        p.push("nvim");
        Ok(p)
    } else {
        let mut p = cu::check!(
            std::env::home_dir(),
            "failed to get user home dir to figure out nvim config dir"
        )?;
        p.extend([".config", "nvim"]);
        Ok(p)
    }
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
    #[serde(default)]
    pub nvim_tree_git: bool,
    #[serde(default = "default_wsclip_host_port")]
    pub wsclip_host_port: u16,
}
fn default_wsclip_host_port() -> u16 {
    8881
}
impl Config {
    pub fn to_lua(&self) -> String {
        format!(
            r##"
-- Config generated by shaft
local M = {{
    nvim_tree_git = {nvim_tree_git},
    editorapi_ssh_wsclip_port = {wsclip_host_port},
}}
return M
        "##,
            nvim_tree_git = self.nvim_tree_git,
            wsclip_host_port = self.wsclip_host_port,
        )
    }
}
