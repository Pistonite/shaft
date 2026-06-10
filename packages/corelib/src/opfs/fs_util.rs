use std::io::Read;
use std::path::Path;
#[cfg(windows)]
use std::path::PathBuf;
use std::sync::Arc;

use cu::pre::*;
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};

#[cfg(windows)]
use crate::opfs;

/// Set the file at path to be executable
#[cfg(windows)]
pub fn set_executable(_: &Path) -> cu::Result<()> {
    Ok(())
}

/// Set the file at path to be executable
#[cfg(not(windows))]
pub fn set_executable(path: &Path) -> cu::Result<()> {
    cu::trace!("setting executable bit for: '{}'", path.display());
    use std::os::unix::fs::PermissionsExt;
    let metadata = path.metadata()?;
    let mut perms = metadata.permissions();
    let mode = perms.mode();
    // Add execute permission for each role that has read permission
    perms.set_mode(mode | ((mode & 0o444) >> 2));
    std::fs::set_permissions(path, perms)?;
    Ok(())
}

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
#[cfg(windows)]
#[cu::context("failed to create hard links")]
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

/// Create hardlinks. `from` is where the link will be and `to` is the target of the link
#[cfg(not(windows))]
#[cu::context("failed to create hard links")]
pub fn hardlink_files(paths: &[(&Path, &Path)]) -> cu::Result<()> {
    for (from, to) in paths {
        cu::fs::remove(from)?;
        std::fs::hard_link(to, from)?;
    }
    Ok(())
}

#[cfg(windows)]
#[cu::context("failed to remove: '{}'", path.display())]
pub fn safe_remove_link(path: &Path) -> cu::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    // it's faster to remove directly, but it might fail
    // if the executable is currently running.
    // However, PowerShell can still remove it as long as
    // the actual inode is not removed (the last copy of the hard link)
    let Err(e) = cu::fs::remove(path) else {
        return Ok(());
    };
    cu::debug!("failed to remove link: {e}, falling back to use powershell");
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
        .stderr(cu::lv::D)
        .stdin_null()
        .wait_nz()?;
    Ok(())
}

#[cfg(not(windows))]
pub fn safe_remove_link(path: &Path) -> cu::Result<()> {
    cu::fs::remove(path)
}

/// Get the SHA256 checksum of a file and return it as a string
#[cu::context("failed to hash file: '{}'", path.display())]
pub fn file_sha256(path: &Path, bar: Option<Arc<cu::ProgressBar>>) -> cu::Result<String> {
    #[cfg(unix)]
    use std::os::unix::fs::MetadataExt;
    #[cfg(windows)]
    use std::os::windows::fs::MetadataExt;

    let mut hasher = Sha256::new();
    let mut reader = cu::fs::reader(path)?;

    let metadata = path.metadata()?;
    #[cfg(unix)]
    let file_size = metadata.size();
    #[cfg(windows)]
    let file_size = metadata.file_size();

    let mut buf = vec![0u8; 4096000].into_boxed_slice();
    let mut current_size = 0;
    loop {
        let i = reader.read(&mut buf)?;
        current_size += i as u64;
        if let Some(bar) = &bar {
            cu::progress!(
                bar,
                "hashing: {:.02}%",
                current_size as f64 * 100.0 / file_size as f64
            );
        }
        if i == 0 {
            break;
        }
        hasher.update(&buf[..i]);
    }
    if let Some(bar) = &bar {
        cu::progress!(bar, "hashing: done");
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

/// Decompress GZ bytes into a file
#[cu::context("failed to unpack gzip bytes")]
pub fn ungz_bytes(bytes: &[u8], out_path: &Path) -> cu::Result<()> {
    let mut decoder = GzDecoder::new(bytes);
    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf)?;
    cu::fs::write(out_path, buf)
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
        if s.contains('"') {
            cu::bail!("quote (\") in path is not allowed: {}", path.display());
        }
        Ok(format!("\"{s}\""))
    }
}

/// Write content to a file using sudo, creating parent directories if needed.
#[inline(always)]
pub fn sudo_write(path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> cu::Result<()> {
    sudo_write_impl(path.as_ref(), content.as_ref())
}
#[cu::context("failed to sudo write: '{}'", path.display())]
fn sudo_write_impl(path: &Path, content: &[u8]) -> cu::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            super::sudo("mkdir", "create parent directories")?
                .args(["-p", parent.as_utf8()?])
                .all_null()
                .wait_nz()?;
        }
    }
    super::sudo("tee", &format!("writing {}", path.display()))?
        .arg(path)
        .stdin(cu::pio::write(content.to_vec()))
        .stdout(cu::lv::D)
        .stderr(cu::lv::E)
        .wait_nz()?;
    Ok(())
}

/// Find a file in the Windows installation of Git
#[cfg(windows)]
#[inline(always)]
pub fn find_in_wingit(path: impl AsRef<Path>) -> cu::Result<PathBuf> {
    find_in_wingit_impl(path.as_ref())
}
#[cfg(windows)]
#[cu::context("cannot find in git installation: '{}'", path.display())]
fn find_in_wingit_impl(path: &Path) -> cu::Result<PathBuf> {
    let mut git_path = cu::which("git")?;
    // find the mingw64 or clangarm64 (in arm installation)
    let mut mingw64_path = git_path.join("mingw64");
    let mut clangarm64_path = git_path.join("clangarm64");
    while !mingw64_path.is_dir() && !clangarm64_path.is_dir() {
        git_path = git_path.parent_abs()?;
        mingw64_path = git_path.join("mingw64");
        clangarm64_path = git_path.join("clangarm64");
    }
    if mingw64_path.is_dir() {
        cu::trace!("found mingw64: '{}'", mingw64_path.display());
    }
    if clangarm64_path.is_dir() {
        cu::trace!("found clangarm64: '{}'", clangarm64_path.display());
    }
    git_path.join(path).normalize_executable()
}
