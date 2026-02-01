use std::io::{Cursor, Read};
use std::path::Path;
#[cfg(windows)]
use std::path::PathBuf;
use std::sync::Arc;

use cu::pre::*;
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use tar::Archive as TarArchive;
use zip::ZipArchive;

#[cfg(windows)]
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
        .stderr(cu::lv::E)
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

/// Extract an archive.
///
/// Supports `.tar.gz`, `.tgz` and `.zip`
#[inline(always)]
pub fn unarchive(
    archive_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
    clean: bool,
) -> cu::Result<()> {
    unarchive_impl(archive_path.as_ref(), out_dir.as_ref(), clean)
}
fn unarchive_impl(archive_path: &Path, out_dir: &Path, clean: bool) -> cu::Result<()> {
    let ext = cu::check!(
        archive_path.extension(),
        "missing archive extension: '{}'",
        archive_path.display()
    )?;
    let ext = ext.to_ascii_lowercase();
    let ext = cu::check!(ext.into_utf8(), "unknown archive extension")?;
    enum Format {
        Tar,
        TarGz,
        Zip,
    }
    let format = match ext.as_bytes() {
        b"gz" => {
            let mut path = archive_path.to_path_buf();
            path.set_extension("");
            let ext = cu::check!(
                archive_path.extension(),
                "only .tar.gz is supported with .gz files"
            )?;
            let ext = ext.to_ascii_lowercase();
            if ext != "tar" {
                cu::bail!("only .tar.gz is supported for .gz files");
            }
            Format::TarGz
        }
        b"tgz" => Format::TarGz,
        b"tar" => Format::Tar,
        b"zip" => Format::Zip,
        _ => {
            cu::bail!("unknown archive extension: {ext}")
        }
    };
    let archive_bytes = cu::fs::read(archive_path)?;
    match format {
        Format::TarGz => {
            untargz_bytes(&archive_bytes, out_dir, clean)?;
        }
        Format::Tar => {
            untar_bytes(&archive_bytes, out_dir, clean)?;
        }
        Format::Zip => {}
    }
    Ok(())
}

#[cu::context("failed to unpack targz bytes")]
pub fn untargz_bytes(archive_bytes: &[u8], out_dir: &Path, clean: bool) -> cu::Result<()> {
    if clean {
        cu::fs::make_dir_empty(out_dir)?;
    }
    let mut archive = TarArchive::new(GzDecoder::new(archive_bytes));
    archive.unpack(out_dir)?;
    Ok(())
}

#[cu::context("failed to unpack tar bytes")]
pub fn untar_bytes(archive_bytes: &[u8], out_dir: &Path, clean: bool) -> cu::Result<()> {
    if clean {
        cu::fs::make_dir_empty(out_dir)?;
    }
    let mut archive = TarArchive::new(archive_bytes);
    archive.unpack(out_dir)?;
    Ok(())
}

#[cu::context("failed to unpack zip bytes")]
pub fn unzip_bytes(archive_bytes: &[u8], out_dir: &Path, clean: bool) -> cu::Result<()> {
    if clean {
        cu::fs::make_dir_empty(out_dir)?;
    }
    let mut archive = ZipArchive::new(Cursor::new(archive_bytes))?;
    archive.extract_unwrapped_root_dir(out_dir, zip::read::root_dir_common_filter)?;
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
        if s.contains('"') {
            cu::bail!("quote (\") in path is not allowed: {}", path.display());
        }
        Ok(format!("\"{s}\""))
    }
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
    // find the mingw64
    let mut mingw64_path = git_path.join("mingw64");
    while !mingw64_path.is_dir() {
        git_path = git_path.parent_abs()?;
        mingw64_path = git_path.join("mingw64");
    }
    cu::trace!("found mingw64: '{}'", mingw64_path.display());
    git_path.join(path).normalize_executable()
}
