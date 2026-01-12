//! PowerShell 7

use crate::pre::*;

// using preview version to enable tilde (~) expansion
static VERSION: &str = "7.6.0-preview.6";

register_binaries!("pwsh");

pub fn binary_dependencies() -> EnumSet<BinId> {
    enum_set! { BinId::_7z }
}

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_bin_in_path_and_shaft!("pwsh");
    let version = command_output!("pwsh", ["-NoLogo", "-NoProfile", "-c", "$PSVersionTable.PSVersion.ToString()"]);
    let is_preview = version.contains("preview");
    let is_uptodate = Version(version.trim()) >= VERSION;
    Ok(Verified::is_uptodate(is_preview && is_uptodate))
}
pub fn download(_: &Context) -> cu::Result<()> {
    let sha256_checksum = if cfg!(target_arch = "aarch64") {
        "36dc90e7f0e7870b0970c9a58790de4de4217e65acafaf790e87b7c97d93649f"
    } else {
        "481ce45bd9ebfab9a5b254a35f145fb6259bd452ae67d92ab1d231b6367987d9"
    };
    hmgr::download_file("pwsh.zip", download_url(), sha256_checksum)?;
    Ok(())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    opfs::ensure_terminated("pwsh.exe")?;
    cu::fs::make_dir(hmgr::paths::install_root())?;
    ctx.move_install_to_old_if_exists()?;

    let pwsh_zip = hmgr::paths::download("pwsh.zip", download_url());
    let pwsh_dir = ctx.install_dir();
    opfs::un7z(pwsh_zip, &pwsh_dir)?;

    let pwsh_exe = pwsh_dir.join("pwsh.exe");
    ctx.add_item(hmgr::Item::ShimBin("pwsh".to_string(), 
        vec![
            pwsh_exe.into_utf8()?
        ]
    ))?;
    Ok(())
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    opfs::ensure_terminated("pwsh.exe")?;
    ctx.move_install_to_old_if_exists()?;
    Ok(())
}

fn download_url() -> String {
    let arch = is_arm!("arm64", else "x64");
    format!("https://github.com/PowerShell/PowerShell/releases/download/v{VERSION}/PowerShell-{VERSION}-win-{arch}.zip")
}
