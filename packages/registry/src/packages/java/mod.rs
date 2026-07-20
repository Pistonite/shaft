//! Jabba-shim, Java Version Manager shimmed
use crate::pre::*;

register_binaries!("java", "javac", "jabba");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_cargo!("jabba" in crate "jabba-shim");
    check_in_path!("java");
    check_in_path!("javac");

    let version = current_version()?;
    check_outdated!(&version, metadata[java]::VERSION);
    Ok(Verified::UpToDate)
}

fn current_version() -> cu::Result<String> {
    let version = command_output!("jabba", ["-vV"]);
    for l in version.lines() {
        if let Some(v) = l.strip_prefix("jabba-shim ") {
            return Ok(v.trim().to_string());
        }
    }
    Ok("unknown".to_string())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::cargo::binstall_git("jabba-shim", metadata::java::REPO, ctx.bar_ref())?;
    Ok(())
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    epkg::cargo::uninstall("jabba-shim")?;
    if let Err(e) = ctx.move_install_to_old_if_exists() {
        cu::warn!("failed to remove installed jdks: {e:?}");
    }
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let jabba_home = ctx.install_dir();
    ctx.add_item(Item::user_env_var("JABBA_HOME", jabba_home.as_utf8()?))?;
    let java_home = if cfg!(target_os = "macos") {
        cu::path!(&jabba_home / "jdk" / "current" / "Contents" / "Home")
    } else {
        cu::path!(&jabba_home / "jdk" / "current")
    };
    ctx.add_item(Item::user_env_var("JAVA_HOME", java_home.as_utf8()?))?;
    let java_home_bin = cu::path!(java_home / "bin");
    ctx.add_item(Item::user_path(java_home_bin.into_utf8()?))?;

    let config = ctx.load_config(CONFIG)?;
    if cfg!(windows) {
        cu::warn!("jabba in Windows requires Developer Mode to create symlinks");
    }
    let (child, bar, _) = cu::which("jabba")?
        .command()
        .env("JABBA_HOME", &jabba_home)
        .args(["install", &config.default_jdk])
        .stdoe(cu::pio::spinner("install java").configure_spinner(|b| b.parent(ctx.bar())))
        .stdin_null()
        .spawn()?;
    child.wait_nz()?;
    bar.done();
    cu::which("jabba")?
        .command()
        .args(["use", &config.default_jdk])
        .env("JABBA_HOME", &jabba_home)
        .stdoe(cu::lv::P)
        .stdin_null()
        .wait_nz()?;
    Ok(())
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
    pub default_jdk: String,
}
