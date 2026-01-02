
//! Git version control System

use cu::pre::*;
use op::installer::pacman;

use crate::pre::*;

register_binaries!("git");

static GIT_WINDOWS_VERSION: &str = "2.51.2.windows.1";
static GIT_MICROSOFT_VERSION: &str = "2.51.2.vfs";
static GIT_WINDOWS_ARM_SHA256: &str = "cfa59dc9ca121844a9346224e856ee11916ebd606b211d4291f8b97aa482dd94";
static GIT_WINDOWS_X64_SHA256: &str = "ebd318e1d3ee0cc1ac8ead026f1edf8678dcb42c7d74d757b8e2fa8a1be0b25f";

pub fn verify(ctx: &Context) -> cu::Result<Verified> {
    check_bin_in_path!("git");

    // portable git
    let expected = op::home::bin("git.exe");
    cu::check!(ctx.check_bin_location("git", &expected),
        "current 'git' is not installed with shaft; please uninstall it or use the 'system-git' package")?;
    version::verify(ctx)
}

pub async fn download(ctx: &Context) -> cu::Result<()> {
    let temp_dir = ctx.temp_dir();
    let download_path = temp_dir.join("git.7z.exe");
    let extract_path = temp_dir.join("extracted");
    let url = version::windows_download_url();
    // download
let sha256 = if op::is_arm!() {
        GIT_WINDOWS_ARM_SHA256
    } else {
        GIT_WINDOWS_X64_SHA256
    };
    op::co_download_to_file(url, &download_path, sha256).await?;
    // extract
    download_path.command()
        .add(cu::args!["-o", extract_path, "-y"])
        .all_null()
        .co_wait_nz().await?;
    Ok(())
}
fn download_url() -> String {
    let i = match GIT_WINDOWS_VERSION.find("windows"){
        Some(x) => x-1,
        None => GIT_WINDOWS_VERSION.len()
    };
    let version_short = &GIT_WINDOWS_VERSION[0..i];
    let arch = if op::is_arm!() {
        "arm64"
    } else {
        "64-bit"
    };

    format!("https://github.com/git-for-windows/git/releases/download/v{GIT_WINDOWS_VERSION}/PortableGit-{version_short}-{arch}.7z.exe")
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    // match ctx.platform {
    //     Platform::Arch => {
    //         op::sysinfo::ensure_terminated("git")?;
    //         todo!()
    //     }
    //     Platform::Windows => {
    //         op::sysinfo::ensure_terminated("git.exe")?;
    //         let temp_dir = ctx.temp_dir();
    //         let extract_path = temp_dir.join("extracted");
    //         let old_path = temp_dir.join("old");
    //         cu::fs::rec_remove(&old_path)?;
    //         let install_dir = ctx.install_dir();
    //         if install_dir.exists() {
    //             std::fs::rename(&install_dir, old_path)
    //         }
    //     }
    // }
    todo!()
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    todo!()
}

pub mod version;
