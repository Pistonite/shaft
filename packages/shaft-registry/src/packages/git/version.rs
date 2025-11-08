use cu::pre::*;

use crate::pre::*;

static GIT_WINDOWS_VERSION: &str = "2.51.2.windows.1";
static GIT_WINDOWS_ARM_SHA256: &str = "cfa59dc9ca121844a9346224e856ee11916ebd606b211d4291f8b97aa482dd94";
static GIT_WINDOWS_X64_SHA256: &str = "ebd318e1d3ee0cc1ac8ead026f1edf8678dcb42c7d74d757b8e2fa8a1be0b25f";
static GIT_MICROSOFT_VERSION: &str = "2.51.2.vfs";
static GIT_VERSION: &str = "2.51.2";

pub fn verify(ctx: &Context) -> cu::Result<Verified> {
    let Ok(git) = cu::which("git") else {
        return Ok(Verified::NotInstalled);
    };
    let (child, stdout) = git
        .command()
        .arg("--version")
        .stdout(cu::pio::string())
        .stdie_null()
        .spawn()?;
    child.wait_nz()?;
    let stdout = stdout.join()??;
    let version = stdout.strip_prefix("git version ").unwrap_or(&stdout);

    if ctx.platform != Platform::Windows {
        if version.is_version_same_or_higher_than(GIT_VERSION) {
            return Ok(Verified::UpToDate);
        }
        return Ok(Verified::NotUpToDate);
    }
    let min_version = if version.contains("vfs") {
        GIT_MICROSOFT_VERSION
    } else {
        GIT_WINDOWS_VERSION
    };
    if version.is_version_same_or_higher_than(min_version) {
        Ok(Verified::UpToDate)
    }
    else {
        Ok(Verified::NotUpToDate)
    }
}

pub fn windows_download_url() -> String {
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

pub fn windows_sha256() -> &'static str {
    if op::is_arm!() {
        GIT_WINDOWS_ARM_SHA256
    } else {
        GIT_WINDOWS_X64_SHA256
    }
}
