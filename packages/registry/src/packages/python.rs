//! UV, `python` manager by Astro
use crate::pre::*;

register_binaries!("uv", "uvx", "python");

pub fn verify(_: &Context) -> cu::Result<Verified> {
    if cfg!(windows) {
        check_in_shaft!("python");
    } else {
        // python might be shipped with other OS
        check_in_path!("python");
    }
    let v = check_cargo!("uv");
    check_outdated!(&v.version, metadata[uv]::VERSION);
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::cargo::binstall("uv", ctx.bar_ref())?;
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    epkg::cargo::uninstall("uv")?;
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let install_dir = ctx.install_dir();
    cu::fs::make_dir(&install_dir)?;
    let python_install_dir = install_dir.join("python");
    let python_cache_dir = install_dir.join("python-cache");
    let tool_dir = install_dir.join("tool");
    let cache_dir = install_dir.join("cache");
    let bin_dir = hmgr::paths::bin_root();

    let env_map = [
        ("UV_CACHE_DIR".to_string(), cache_dir.into_utf8()?),
        (
            "UV_PYTHON_INSTALL_DIR".to_string(),
            python_install_dir.into_utf8()?,
        ),
        (
            "UV_PYTHON_BIN_DIR".to_string(),
            bin_dir.clone().into_utf8()?,
        ),
        (
            "UV_PYTHON_CACHE_DIR".to_string(),
            python_cache_dir.into_utf8()?,
        ),
        ("UV_TOOL_DIR".to_string(), tool_dir.into_utf8()?),
        ("UV_TOOL_BIN_DIR".to_string(), bin_dir.into_utf8()?),
    ];

    // install latest python and set it as default
    {
        let (child, bar, _) = cu::which("uv")?
            .command()
            .args(["python", "install", "--default"])
            .envs(env_map.clone())
            .stdoe(cu::pio::spinner("install python").configure_spinner(|b| b.parent(ctx.bar())))
            .stdin_null()
            .spawn()?;
        child.wait_nz()?;
        bar.done();
    }

    // zap env var
    for (key, value) in env_map {
        ctx.add_item(Item::user_env_var(key, value))?;
    }

    Ok(())
}
