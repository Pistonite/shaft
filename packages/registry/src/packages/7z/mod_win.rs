//! 7-Zip
use crate::pre::*;

mod version;

register_binaries!("7z", "7zfm");

static VERSION: &str = "25.01";

pub fn verify(ctx: &Context) -> cu::Result<Verified> {
    check_bin_in_path_and_shaft!("7z");
    check_bin_in_path_and_shaft!("7zfm");
    if !uninstaller_path(ctx).exists() {
        // ensure the uninstaller exists in a good installation
        // so it can be uninstalled
        return Ok(Verified::NotUpToDate);
    }
    version::check(VERSION)
}
pub fn download(ctx: &Context) -> cu::Result<()> {
    let sha256_checksum = 
    is_arm!(
"6365c7c44e217b9c1009e065daf9f9aa37454e64315b4aaa263f7f8f060755dc",
else 
"78afa2a1c773caf3cf7edf62f857d2a8a5da55fb0fff5da416074c0d28b2b55f");
    hmgr::download_file("7z-installer.exe", download_url(), sha256_checksum, ctx.bar())?;
    Ok(())
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    ensure_terminated()?;
    cu::fs::make_dir(hmgr::paths::install_root())?;
    ctx.move_install_to_old_if_exists()?;

    let installer = hmgr::paths::download("7z-installer.exe", download_url());
    // https://7-zip.org/faq.html
    // /S is silent, /D specify install dir
    let install_dir = ctx.install_dir();
    let script = format!("{} /S /D={}", installer.as_utf8()?, opfs::quote_path(&install_dir)?);
    opfs::sudo("powershell", "7z installer")?
        .args(["-NoLogo", "-NoProfile", "-c", &script])
        .stdoe(cu::lv::D)
        .stdin_null()
        .wait_nz()?;

    cu::fs::make_dir(hmgr::paths::bin_root())?;
    let exe_path = hmgr::paths::binary("7z.exe");
    let exefm_path = hmgr::paths::binary("7zfm.exe");
    ctx.add_item(hmgr::Item::LinkBin(
        exe_path.into_utf8()?, 
        install_dir.join("7z.exe").into_utf8()?
    ))?;
    ctx.add_item(hmgr::Item::LinkBin(
        exefm_path.into_utf8()?, 
        install_dir.join("7zFM.exe").into_utf8()?
    ))?;
    Ok(())
}
pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    ensure_terminated()?;

    let uninstaller = uninstaller_path(ctx);
    if !uninstaller.exists() {
        cu::bail!("cannot find 7zip uninstaller, please sync it first to repair the installation");
    }
    opfs::sudo_path(&uninstaller, "7z uninstaller")?
    .args(["/S"]) // silent
        .stdoe(cu::lv::D)
        .stdin_null()
        .wait_nz()?;

    let exe_path = hmgr::paths::binary("7z.exe");
    cu::fs::remove(&exe_path)?;
    let exefm_path = hmgr::paths::binary("7zfm.exe");
    cu::fs::remove(&exefm_path)?;

    // leftover reg entry
    // HKEY_CLASSES_ROOT/*/shellex/ContextMenuHandlers/7-Zip
    // but it's not a big problem
    Ok(())
}

fn ensure_terminated() -> cu::Result<()> {
    opfs::ensure_terminated("7z.exe")?;
    opfs::ensure_terminated("7zFM.exe")?;
    Ok(())
}

fn download_url() -> String {
    let arch = is_arm!("arm64", else "x64");
    let version_no_dot = VERSION.replace(".", "");
    format!("https://github.com/ip7z/7zip/releases/download/{VERSION}/7z{version_no_dot}-{arch}.exe")
}

fn uninstaller_path(ctx: &Context) -> PathBuf {
    ctx.install_dir().join("Uninstall.exe")
}
