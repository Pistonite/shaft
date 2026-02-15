use std::io::{BufRead, Cursor, Read};
use std::path::Path;
#[cfg(windows)]
use std::path::PathBuf;
use std::sync::Arc;

use cu::pre::*;
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use tar::Archive as TarArchive;
use xz2::bufread::XzDecoder;
use zip::ZipArchive;

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
    cu::trace!(
        "extracting '{}' to '{}', clean={}",
        archive_path.display(),
        out_dir.display(),
        clean
    );
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
        TarXz,
        Zip,
        Use7z,
    }
    let file_size = {
        let metadata = archive_path.metadata()?;
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;
        #[cfg(windows)]
        use std::os::windows::fs::MetadataExt;
        #[cfg(unix)]
        let file_size = metadata.size();
        #[cfg(windows)]
        let file_size = metadata.file_size();
        file_size
    };
    let is_big_file = file_size >= 50_000_000;
    let format = match ext.as_bytes() {
        b"gz" => Format::TarGz, // assume tar.gz
        b"xz" => Format::TarXz, // assume tar.xz
        b"tgz" => Format::TarGz,
        b"txz" => Format::TarXz,
        b"tar" => {
            if is_big_file {
                Format::Use7z
            } else {
                Format::Tar
            }
        }
        b"zip" => {
            if is_big_file {
                Format::Use7z
            } else {
                Format::Zip
            }
        }
        _ => {
            cu::debug!("unsupported archive extension: {ext}, trying to spawn 7z to deal with it");
            Format::Use7z
        }
    };
    match format {
        Format::TarGz => {
            let mut archive_bytes = cu::fs::reader(archive_path)?;
            untargz_read(&mut archive_bytes, out_dir, clean)?;
        }
        Format::TarXz => {
            let mut archive_bytes = cu::fs::reader(archive_path)?;
            untarxz_read(&mut archive_bytes, out_dir, clean)?;
        }
        Format::Tar => {
            let mut archive_bytes = cu::fs::reader(archive_path)?;
            untar_read(&mut archive_bytes, out_dir, clean)?;
        }
        Format::Zip => {
            let archive_bytes = cu::fs::read(archive_path)?;
            unzip_bytes(&archive_bytes, out_dir, clean)?;
        }
        Format::Use7z => {
            let exe = cu::which("7z")?;
            let command = if cfg!(windows) {
                // on windows, spawning 7z directly is diffcult
                // to do path escape, so we wrap it with powershell
                let script = format!(
                    "& {} x -y {} -o{}",
                    quote_path(&exe)?,
                    quote_path(archive_path)?,
                    quote_path(out_dir)?,
                );
                cu::which("powershell.exe")?
                    .command()
                    .args(["-NoLogo", "-c", &script])
            } else {
                let out_dir = quote_path(out_dir)?;
                cu::which("7z")?.command().add(cu::args![
                    "x",
                    "-y",
                    archive_path,
                    format!("-o{out_dir}")
                ])
            };
            let (child, bar, _) = command
                .stdoe(cu::pio::spinner("7z").configure_spinner(|x| x.keep(false)))
                .stdin_null()
                .spawn()?;
            child.wait_nz()?;
            bar.done();
        }
    }
    Ok(())
}

#[cu::context("failed to unpack targz bytes")]
pub fn untargz_read(archive_bytes: impl BufRead, out_dir: &Path, clean: bool) -> cu::Result<()> {
    untar_read(GzDecoder::new(archive_bytes), out_dir, clean)
}

#[cu::context("failed to unpack tarxz bytes")]
pub fn untarxz_read(archive_bytes: impl BufRead, out_dir: &Path, clean: bool) -> cu::Result<()> {
    untar_read(XzDecoder::new(archive_bytes), out_dir, clean)
}

#[cu::context("failed to unpack tar bytes")]
pub fn untar_read(archive_bytes: impl Read, out_dir: &Path, clean: bool) -> cu::Result<()> {
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
    archive.extract(out_dir)?; // to be consistent with 7z, we do not unwrap root dir
    Ok(())
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
    // find the mingw64
    let mut mingw64_path = git_path.join("mingw64");
    while !mingw64_path.is_dir() {
        git_path = git_path.parent_abs()?;
        mingw64_path = git_path.join("mingw64");
    }
    cu::trace!("found mingw64: '{}'", mingw64_path.display());
    git_path.join(path).normalize_executable()
}
