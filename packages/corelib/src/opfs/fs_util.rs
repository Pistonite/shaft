use std::io::Read;
use std::path::Path;

use cu::pre::*;
use sha2::{Digest, Sha256};

use crate::opfs;

/// Create a Windows symbolic link (requires sudo).
/// `from` is where the link will be
#[cfg(windows)]
#[cu::context("failed to create symbolic links")]
pub fn symlink_files(paths: &[(&Path, &Path)]) -> cu::Result<()> {
    let mut script = String::new();
    for (from, to) in paths {
        cu::fs::remove(from)?;
        let from_abs = from.normalize()?;
        let to_abs = to.normalize()?;

        let from_str = from_abs.as_utf8()?;
        let to_str = to_abs.as_utf8()?;
        build_link_powershell(&mut script, "SymbolicLink", from_str, to_str);
    }
    // use powershell since sudo is required
    opfs::sudo("powershell", "create symlinks")?
        .args(["-NoLogo", "-NoProfile", "-c", &script])
        .stdout(cu::lv::D)
        .stderr(cu::lv::E)
        .stdin_null()
        .wait_nz()?;

    Ok(())
}

/// Create hardlinks. `from` is where the link will be and `to` is the target of the link
pub fn hardlink_files(paths: &[(&Path, &Path)]) -> cu::Result<()> {
    let mut script = String::new();
    for (from, to) in paths {
        safe_remove_link(from)?;
        let from_abs = from.normalize()?;
        let to_abs = to.normalize()?;
        let from_str = from_abs.as_utf8()?;
        let to_str = to_abs.as_utf8()?;
        build_link_powershell(&mut script, "HardLink", from_str, to_str);
    }
    cu::which("powershell")?
        .command()
        .args(["-NoLogo", "-NoProfile", "-c", &script])
        .stdout(cu::lv::D)
        .stderr(cu::lv::E)
        .stdin_null()
        .wait_nz()?;

    Ok(())
}

#[cfg(windows)]
fn build_link_powershell(out: &mut String, link_type: &str, from: &str, to: &str) {
    out.push_str(&format!(
        "New-Item -ItemType {link_type} -Path \"{from}\" -Target \"{to}\";"
    ))
}

#[cfg(windows)]
#[cu::context("failed to remove: '{}'", path.display())]
pub fn safe_remove_link(path: &Path) -> cu::Result<()> {
    // remove with powershell
    cu::which("powershell")?
        .command()
        .args([
            "-NoLogo",
            "-NoProfile",
            "-c",
            &format!("Remove-Item {}", quote_path(path)?),
        ])
        .stdout(cu::lv::D)
        .stderr(cu::lv::E)
        .stdin_null()
        .wait_nz()?;
    Ok(())
}

/// Get the SHA256 checksum of a file and return it as a string
#[cu::context("failed to hash file: '{}'", path.display())]
pub fn file_sha256(path: &Path) -> cu::Result<String> {
    let mut hasher = Sha256::new();
    let mut reader = cu::fs::reader(&path)?;
    let mut buf = vec![0u8; 4096000].into_boxed_slice();
    loop {
        let i = reader.read(&mut buf)?;
        if i == 0 {
            break;
        }
        hasher.update(&buf[..i]);
    }
    let result = hasher.finalize();
    let mut out = String::with_capacity(64);
    let digits = b"0123456789abcdef";
    for b in result {
        let c1 = digits[(b / 16) as usize] as char;
        let c2 = digits[(b % 16) as usize] as char;
        out.push(c1);
        out.push(c2);
    }
    Ok(out)
}

/// Extract an archive with 7z. Requires the 7z binary to exist
#[inline(always)]
pub fn un7z(archive_path: impl AsRef<Path>, out_dir: impl AsRef<Path>) -> cu::Result<()> {
    un7z_impl(archive_path.as_ref(), out_dir.as_ref())
}

#[cu::context("failed to extract zip: '{}'", archive_path.display())]
fn un7z_impl(archive_path: &Path, out_dir: &Path) -> cu::Result<()> {
    let script = format!(
        "& {} x -y {} -o{}",
        quote_path(cu::which("7z")?)?,
        quote_path(archive_path)?,
        quote_path(out_dir)?
    );
    // 7z will create the out dir if not exist, so we don't need to check

    cu::which("powershell")?
        .command()
        .args(["-NoLogo", "-NoProfile", "-c", &script])
        .stdoe(cu::lv::D)
        .stdin_null()
        .wait_nz()?;
    Ok(())
}

/// Ensure nothing weird happens when the path is quoted
#[inline(always)]
pub fn quote_path(path: impl AsRef<Path>) -> cu::Result<String> {
    quote_path_impl(path.as_ref())
}
fn quote_path_impl(path: &Path) -> cu::Result<String> {
    if cfg!(windows) {
        // quote cannot be in the path on Windows
        Ok(format!("\"{}\"", path.as_utf8()?))
    } else {
        let s = path.as_utf8()?;
        cu::ensure!(
            !s.contains('"'),
            "quote (\") in path is not allowed: {}",
            path.display()
        );
        Ok(format!("\"{s}\""))
    }
}
