//! PowerShell 7

use crate::pre::*;

register_binaries!("pwsh");

pub fn config_dependencies() -> EnumSet<PkgId> {
    enum_set! { PkgId::Shellutils }
}

pub fn verify(ctx: &Context) -> cu::Result<Verified> {
    match cu::which("pwsh") {
        Err(_) => return Ok(Verified::NotInstalled),
        Ok(path) => {
            if path != hmgr::paths::binary(bin_name!("pwsh"))
            // sometimes inside the shell, the pwsh executable can point
            // to the real executable, so we need this extra check
            && path != ctx.install_dir().join(bin_name!("pwsh"))
            {
                cu::bail!(
                    "found existing '{}' installed outside of shaft, please uninstall it first (at '{}')",
                    "pwsh",
                    path.display()
                );
            }
        }
    }
    let version = command_output!(
        "pwsh",
        [
            "-NoLogo",
            "-NoProfile",
            "-c",
            "$PSVersionTable.PSVersion.ToString()"
        ]
    );
    let is_preview = version.contains("preview");
    let is_uptodate = !(Version(version.trim()).lt(metadata::pwsh::VERSION));
    Ok(Verified::is_uptodate(is_preview && is_uptodate))
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file("pwsh.zip", download_url(), metadata::pwsh::SHA, ctx.bar())?;
    Ok(())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    opfs::ensure_terminated("pwsh.exe")?;
    ctx.move_install_to_old_if_exists()?;

    let pwsh_zip = hmgr::paths::download("pwsh.zip", download_url());
    let pwsh_dir = ctx.install_dir();
    opfs::unarchive(pwsh_zip, &pwsh_dir, true)?;

    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let pwsh_exe = ctx.install_dir().join("pwsh.exe");
    ctx.add_item(Item::shim_bin(
        bin_name!("pwsh"),
        ShimCommand::target(pwsh_exe.as_utf8()?),
    ))?;
    // get ps7 profile location
    let (child, stdout) = pwsh_exe
        .command()
        .args(["-NoLogo", "-NoProfile", "-c", "$Profile.AllUsersAllHosts"])
        .stdout(cu::pio::string())
        .stderr(cu::lv::E)
        .stdin_null()
        .spawn()?;
    child.wait_nz()?;
    let ps7_profile_path = Path::new(stdout.join()??.trim()).normalize()?;
    let mut edit_profile_path = ps7_profile_path.clone();
    let config = ctx.load_config_file_or_default(include_str!("config.toml"))?;
    if let Some(toml::Value::String(ps5_profile)) = config.get("use-ps5-profile") {
        if !matches!(
            ps5_profile.as_str(),
            "AllUsersAllHosts"
                | "AllUsersCurrentHost"
                | "CurrentUserAllHosts"
                | "CurrentUserCurrentHost"
        ) {
            cu::bail!("invalid powershell profile name: {ps5_profile}");
        }
        // get ps5 profile location
        let (child, stdout) = cu::which("powershell.exe")?
            .command()
            .args([
                "-NoLogo",
                "-NoProfile",
                "-c",
                &format!("$Profile.{ps5_profile}"),
            ])
            .stdout(cu::pio::string())
            .stderr(cu::lv::E)
            .stdin_null()
            .spawn()?;
        child.wait_nz()?;
        let ps5_profile_path = Path::new(stdout.join()??.trim()).normalize()?;
        if let Ok(ps5_profile_content) = cu::fs::read_string(&ps5_profile_path) {
            cu::fs::write(ps7_profile_path, ps5_profile_content)?;
        }
        edit_profile_path = ps5_profile_path;
    }

    if ctx.is_installed(PkgId::Shellutils) {
        ctx.add_item(Item::shim_bin(
            bin_name!("vipwsh"),
            ShimCommand::target_args(
                cu::which("viopen")?.into_utf8()?,
                [edit_profile_path.into_utf8()?],
            ),
        ))?;
    }
    Ok(())
}

pub fn config_location(ctx: &Context) -> cu::Result<Option<PathBuf>> {
    Ok(Some(ctx.config_file()))
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    opfs::ensure_terminated("pwsh.exe")?;
    ctx.move_install_to_old_if_exists()?;
    Ok(())
}

fn download_url() -> String {
    let repo = metadata::pwsh::REPO;
    let arch = if_arm!("arm64", else "x64");
    let version = metadata::pwsh::VERSION;
    format!("{repo}/releases/download/v{version}/PowerShell-{version}-win-{arch}.zip")
}
