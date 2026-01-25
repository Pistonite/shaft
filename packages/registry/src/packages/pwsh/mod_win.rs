//! PowerShell 7

use crate::pre::*;

// using preview version to enable tilde (~) expansion
static VERSION: Version = Version("7.6.0-preview.6");

register_binaries!("pwsh");

pub fn binary_dependencies() -> EnumSet<BinId> {
    enum_set! { BinId::_7z }
}

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_bin_in_path_and_shaft!("pwsh");
    let version = command_output!("pwsh", ["-NoLogo", "-NoProfile", "-c", "$PSVersionTable.PSVersion.ToString()"]);
    let is_preview = version.contains("preview");
    let is_uptodate = VERSION <= version.trim();
    Ok(Verified::is_uptodate(is_preview && is_uptodate))
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    let sha256_checksum = if_arm!(
        "36dc90e7f0e7870b0970c9a58790de4de4217e65acafaf790e87b7c97d93649f"
    , else 
        "481ce45bd9ebfab9a5b254a35f145fb6259bd452ae67d92ab1d231b6367987d9"
    );
    hmgr::download_file("pwsh.zip", download_url(), sha256_checksum, ctx.bar())?;
    Ok(())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    opfs::ensure_terminated("pwsh.exe")?;
    ctx.move_install_to_old_if_exists()?;

    let pwsh_zip = hmgr::paths::download("pwsh.zip", download_url());
    let pwsh_dir = ctx.install_dir();
    opfs::un7z(pwsh_zip, &pwsh_dir)?;

    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let pwsh_exe = ctx.install_dir().join("pwsh.exe");
    ctx.add_item(hmgr::Item::ShimBin("pwsh.exe".to_string(), 
        vec![
            pwsh_exe.as_utf8()?.to_string()
        ]
    ))?;
    let config = ctx.load_config_file_or_default(include_str!("config.toml"))?;
    if let Some(toml::Value::String(ps5_profile)) = config.get("use-ps5-profile") {
        if !matches!(ps5_profile.as_str(),
            "AllUsersAllHosts" | "AllUsersCurrentHost"
            | "CurrentUserAllHosts" | "CurrentUserCurrentHost") {
            cu::bail!("invalid powershell profile name: {ps5_profile}");
        }
        // get ps5 profile location
        let (child, stdout) = cu::which("powershell.exe")?.command()
            .args(["-NoLogo", "-NoProfile", "-c", &format!("$Profile.{ps5_profile}")])
            .stdout(cu::pio::string())
            .stderr(cu::lv::E)
            .stdin_null()
            .spawn()?;
        child.wait_nz()?;
        let ps5_profile_path = Path::new(stdout.join()??.trim()).normalize()?;
        if let Ok(ps5_profile_content) = cu::fs::read_string(ps5_profile_path) {
            // get ps7 profile location
            let (child, stdout) = pwsh_exe.command()
                .args(["-NoLogo", "-NoProfile", "-c", "$Profile.AllUsersAllHosts"])
                .stdout(cu::pio::string())
                .stderr(cu::lv::E)
                .stdin_null()
                .spawn()?;
            child.wait_nz()?;
            let ps7_profile_path = Path::new(stdout.join()??.trim()).normalize()?;
            cu::fs::write(ps7_profile_path, ps5_profile_content)?;
        }
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
    let arch = if_arm!("arm64", else "x64");
    format!("https://github.com/PowerShell/PowerShell/releases/download/v{VERSION}/PowerShell-{VERSION}-win-{arch}.zip")
}
