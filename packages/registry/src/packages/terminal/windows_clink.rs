use crate::pre::*;

version_cache!(static WRAPPER_VERSION = metadata::terminal::clink::WRAPPER_VERSION);

pub fn verify(ctx: &Context) -> cu::Result<Verified> {
    check_in_shaft!("clink-cmd");
    let clink_bat = clink_dir(ctx).join("clink.bat");
    if !clink_bat.exists() {
        return Ok(Verified::NotInstalled);
    }
    let clink_bat = clink_bat.into_utf8()?;
    let v = command_output!("cmd", ["/c", &clink_bat, "--version"]);
    check_outdated!(&v, metadata[terminal::clink]::VERSION);

    check_version_cache!(WRAPPER_VERSION);
    Ok(Verified::UpToDate)
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file(
        "clink.zip",
        metadata::terminal::clink::URL,
        metadata::terminal::clink::SHA,
        ctx.bar(),
    )?;
    Ok(())
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    let clink_zip = hmgr::paths::download("clink.zip", metadata::terminal::clink::URL);
    opfs::unarchive(clink_zip, clink_dir(ctx), true)?;
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let arch = detect_architecture()?;
    hmgr::tools::ensure_unpacked()?;
    let clink_cmd_build_dir = {
        let mut p = hmgr::paths::tools_root();
        p.extend(["__windows__", "clink-cmd"]);
        p
    };

    let mut cmd = cu::which("cmd.exe")?.into_utf8()?;
    cmd.make_ascii_lowercase();
    let real_cmd = {
        let mut system_root = cu::env_var("SystemRoot")?;
        system_root.make_ascii_lowercase();
        format!("{system_root}\\system32\\cmd.exe")
    };
    if cmd != real_cmd {
        cu::bail!("not compiling clink-cmd because cmd.exe location seems suspicous: {cmd}");
    }

    let clink_exe = clink_dir(ctx)
        .join(format!("clink_{arch}.exe"))
        .into_utf8()?;
    let init_cmd = hmgr::paths::init_cmd().into_utf8()?;

    let clink_cmd_exe = {
        let (child, bar, _) = cu::which("powershell")?
            .command()
            .envs([
                ("CLINK_CMD_COMPILE_ARCH", arch),
                ("CLINK_CMD_COMPILE_CMD_EXECUTABLE", &cmd),
                ("CLINK_CMD_COMPILE_CLINK_EXECUTABLE", &clink_exe),
                ("CLINK_CMD_COMPILE_INIT_CMD", &init_cmd),
                ("CLINK_CMD_COMPILE_PRINT_INSTEAD", "0"),
            ])
            .args(["-NoLogo", "-c", "./build.ps1"])
            .current_dir(&clink_cmd_build_dir)
            .stdoe(
                cu::pio::spinner("compiling clink-cmd")
                    .info()
                    .configure_spinner(|x| x.keep(true).parent(ctx.bar())),
            )
            .stdin_null()
            .spawn()?;
        child.wait_nz()?;
        let output = clink_cmd_build_dir.join("clink-cmd.exe");
        if !output.exists() {
            cu::bail!(
                "failed to build clink-cmd.exe: did not find output at '{}'",
                output.display()
            );
        }
        bar.done();
        output
    };
    let clink_cmd_exe_target = ctx.install_dir().join("clink-cmd.exe");
    cu::fs::copy(clink_cmd_exe, &clink_cmd_exe_target)?;
    ctx.add_item(Item::link_bin(
        hmgr::paths::binary("clink-cmd.exe").into_utf8()?,
        clink_cmd_exe_target.into_utf8()?,
    ))?;

    WRAPPER_VERSION.update()?;
    Ok(())
}

fn clink_dir(ctx: &Context) -> PathBuf {
    let mut p = ctx.install_dir();
    p.push("clink");
    p
}

fn detect_architecture() -> cu::Result<&'static str> {
    if cfg!(target_arch = "aarch64") {
        return Ok("arm64");
    }
    if !cfg!(target_arch = "x86_64") {
        cu::bail!("clink is not supported on the target architecture");
    }
    if cfg!(target_pointer_width = "32") {
        return Ok("x86");
    }
    if cfg!(target_pointer_width = "64") {
        return Ok("x64");
    }
    cu::bail!("clink is not supported on the target pointer width");
}
