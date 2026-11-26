//! Configuration for Hex-Rays IDA Pro

use std::collections::BTreeSet;

use corelib::pyse;

use crate::pre::*;

register_binaries!("hexrays", "hexrays-python", "vihexrays"); // uv creates both python/python3
binary_dependencies!(Uv, Viopen);
version_cache!(static CFG_VERSION = metadata::hexrays::CFG_VERSION);
version_cache!(static THEME_VERSION = metadata::hexrays::catppuccin::VERSION);

static IDA_PRO_VERSION: &str = "9.4";

pub fn verify(_: &Context) -> cu::Result<Verified> {
    cu::check!(
        find_ida_pro(),
        "hexrays-cfg requires IDA Pro {IDA_PRO_VERSION} to be installed"
    )?;
    check_in_shaft!("hexrays");
    check_in_shaft!("hexrays-python");
    check_in_shaft!("vihexrays");
    check_config_version_cache!(CFG_VERSION);
    Ok(Verified::UpToDate)
}
pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file(
        "catppuccin-ida.zip",
        metadata::hexrays::catppuccin::URL,
        metadata::hexrays::catppuccin::SHA,
        ctx.bar(),
    )?;
    Ok(())
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    let install_root = ctx.install_dir();
    let self_plugins = install_root.join("plugins");

    // catppuccin theme
    {
        let cat_zip =
            hmgr::paths::download("catppuccin-ida.zip", metadata::hexrays::catppuccin::URL);
        let plugin_path = self_plugins.join("catppuccin");
        opfs::unarchive(&cat_zip, &plugin_path, true)?;
    }
    Ok(())
}
pub fn configure(ctx: &Context) -> cu::Result<()> {
    let config = ctx.load_config(CONFIG)?;

    let ida_dir = find_ida_pro()?;
    let ida_user_dir = ida_pro_user_dir()?;
    let ida_config = json!({
        "Paths": {
            "ida-install-dir": ida_dir.as_utf8()?.to_string()
        }
    });
    cu::fs::write_json_pretty(cu::path!(&ida_user_dir / "ida-config.json"), &ida_config)?;

    let python_path = ctx.install_dir();
    let python_env = pyse::PythonStandaloneEnvironment::new(
        &python_path,
        "3.14.10",
        vec![],
        "This is a shaft-managed Python standalone environment for IDA. Please use the pipi script to install dependencies".to_string()
    )?;
    python_env.ensure(ctx.bar())?;

    let idapyswitch = ida_dir.join("idapyswitch.exe");
    let python_lib = python_env.python_dynamic_library()?;
    idapyswitch
        .command()
        .arg("--force-path")
        .arg(python_lib)
        .stdin_null()
        .stdout(cu::lv::P)
        .stderr(cu::lv::E)
        .wait_nz()?;

    let ida_bin = ida_dir.join("ida.exe");
    let python_bin = python_env.python_interpreter()?;
    ctx.add_item(Item::shim_bin(
        bin_name!("hexrays"),
        ShimCommand::target(ida_bin.into_utf8()?),
    ))?;
    ctx.add_item(Item::shim_bin(
        bin_name!("hexrays-python"),
        ShimCommand::target(python_bin.into_utf8()?),
    ))?;
    ctx.add_item(Item::shim_bin(
        bin_name!("vihexrays"),
        ShimCommand::target("viopen").args([ctx.install_dir().into_utf8()?]),
    ))?;

    cu::check!(
        apply_plugins(ctx, config.clean_plugins, &["catppuccin"]),
        "failed to apply hexrays plugins!"
    )?;
    THEME_VERSION.update()?;

    CFG_VERSION.update()?;
    Ok(())
}

fn apply_plugins(ctx: &Context, clean: bool, plugins: &[&str]) -> cu::Result<()> {
    let mut seen_themes = BTreeSet::new();
    let mut seen_plugin_files = BTreeSet::new();
    let ida_user_dir = ida_pro_user_dir()?;

    let self_plugins = cu::path!((ctx.install_dir()) / "plugins");
    let target_plugins_dir = cu::path!(&ida_user_dir / "plugins");

    if clean {
        cu::hint!("cleaning hexrays plugins");
        cu::fs::rec_remove(&target_plugins_dir)?;
    }

    for plugin in plugins {
        let plugin_dir = cu::path!(&self_plugins / plugin);
        let theme_dir = plugin_dir.join("themes");
        if theme_dir.exists() {
            // theme should be each in one single directory
            for entry in cu::fs::read_dir(theme_dir)? {
                let entry = entry?;
                let name = entry.file_name().into_utf8()?;
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                if !seen_themes.insert(name.clone()) {
                    cu::bail!("multiple plugins provide the theme '{name}'");
                }
                let target_theme_dir = cu::path!(&ida_user_dir / "themes" / name);
                cu::fs::rec_remove(&target_theme_dir)?;
                cu::fs::rec_copy_inefficiently(&path, &target_theme_dir)?;
                cu::print!("installed hexrays theme '{plugin}/{name}'");
            }
        }
        let script_dir = plugin_dir.join("plugins");
        if script_dir.exists() {
            let mut count = 0;
            for entry in cu::fs::walk(&script_dir)? {
                let entry = entry?;
                let rel_path = entry.rel_path()?;
                let target_path = target_plugins_dir.join(&rel_path).normalize()?;
                if !seen_plugin_files.insert(target_path.clone()) {
                    cu::bail!(
                        "multiple plugins provide the plugin file '{}'",
                        rel_path.display()
                    );
                }
                cu::fs::copy(entry.path(), &target_path)?;
                count += 1;
            }
            cu::print!("installed hexrays plugin '{plugin}': {count} files");
        }
    }

    Ok(())
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    ctx.move_install_to_old_if_exists()?;
    Ok(())
}

fn find_ida_pro() -> cu::Result<PathBuf> {
    let system_drive = opfs::system_drive()?;
    let ida_path = PathBuf::from(format!(
        "{system_drive}\\Program Files\\IDA Professional {IDA_PRO_VERSION}"
    ));
    if !ida_path.is_dir() {
        cu::bail!(
            "IDA Pro {IDA_PRO_VERSION} is not installed: expected installation at '{}'",
            ida_path.display()
        );
    }
    if !ida_path.join("ida.exe").is_file() {
        cu::bail!(
            "IDA Pro installation at '{}' is invalid: ida.exe not found",
            ida_path.display()
        );
    }
    Ok(ida_path)
}

fn ida_pro_user_dir() -> cu::Result<PathBuf> {
    let appdata = opfs::user_appdata()?;
    Ok(cu::path!(appdata / "Hex-Rays" / "IDA Pro"))
}

config_file! {
    static CONFIG: Config = {
        template: include_str!("config.toml"),
        migration: [ ],
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Config {
    pub clean_plugins: bool,
}
