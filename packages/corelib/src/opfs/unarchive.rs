use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use cu::pre::*;

use crate::opfs;

/// Extract an archive to `out_dir`, auto-selecting the tool based on format.
///
/// Tar-based formats (`.tar`, `.tar.gz`/`.tgz`, `.tar.xz`/`.txz`) use the
/// system `tar`. Zip and unknown formats use system `7z` (error if not found).
///
/// `clean=true` wipes `out_dir` before extraction.
#[inline(always)]
pub fn unarchive(
    archive_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
    clean: bool,
) -> cu::Result<()> {
    unarchive_impl(archive_path.as_ref(), out_dir.as_ref(), clean)
}

#[cu::context("failed to extract: '{}'", archive_path.display())]
fn unarchive_impl(archive_path: &Path, out_dir: &Path, clean: bool) -> cu::Result<()> {
    cu::trace!(
        "extracting '{}' to '{}', clean={}",
        archive_path.display(),
        out_dir.display(),
        clean
    );
    let ext = archive_path
        .extension()
        .and_then(|e| e.to_ascii_lowercase().into_utf8().ok())
        .unwrap_or_default();

    // tar-based formats: always use tar. Using 7z for tar.gz/tar.xz on Linux
    // causes a two-step extraction (gz->tar only), not a full extraction.
    match ext.as_bytes() {
        b"gz" | b"tgz" | b"xz" | b"txz" | b"tar" => {
            return imp::unarchive_tar("tar", archive_path, out_dir, clean);
        }
        _ => {}
    }

    // zip / unknown formats: require 7z
    let seven_z = cu::check!(
        cu::which("7z"),
        "7z is required to extract .{ext} archives but was not found"
    )?;
    imp::unarchive_7z(seven_z, archive_path, out_dir, clean)
}

/// Extract an archive to `out_dir`, then rename `from` to `to`.
///
/// Retries the rename up to 3 times (10s, 20s, 30s waits) on failure, to
/// guard against Windows FS timing issues after high disk I/O. When `bar` is
/// provided, attaches a countdown child bar during each retry wait.
#[inline(always)]
pub fn unarchive_rename(
    archive_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
    from: impl AsRef<Path>,
    to: impl AsRef<Path>,
    clean: bool,
    bar: Option<Arc<cu::ProgressBar>>,
) -> cu::Result<()> {
    unarchive_rename_impl(
        archive_path.as_ref(),
        out_dir.as_ref(),
        from.as_ref(),
        to.as_ref(),
        clean,
        bar,
    )
}

#[cu::context("failed to extract and rename: '{}'", archive_path.display())]
fn unarchive_rename_impl(
    archive_path: &Path,
    out_dir: &Path,
    from: &Path,
    to: &Path,
    clean: bool,
    bar: Option<Arc<cu::ProgressBar>>,
) -> cu::Result<()> {
    unarchive(archive_path, out_dir, clean)?;
    if let Err(e) = cu::fs::rename(from, to) {
        let mut success = false;
        cu::warn!("rename after extraction failed: {e:?}");
        for i in 1u64..=3 {
            let retry_secs = i * 10;
            match &bar {
                Some(bar) => {
                    let retry_bar = bar
                        .child(format!("retrying after {retry_secs} seconds"))
                        .total(retry_secs as usize)
                        .eta(false)
                        .percentage(false)
                        .spawn();
                    for _ in 0..retry_secs {
                        std::thread::sleep(Duration::from_secs(1));
                        cu::progress!(retry_bar += 1);
                    }
                }
                _ => {
                    std::thread::sleep(Duration::from_secs(retry_secs));
                }
            }
            match cu::fs::rename(from, to) {
                Ok(()) => {
                    success = true;
                    break;
                }
                Err(e) => {
                    cu::error!("[retry #{i}] rename after extraction failed: {e:?}");
                }
            }
        }
        if !success {
            cu::bail!("rename after extraction failed; please see errors above");
        }
    }
    Ok(())
}

#[doc(hidden)]
pub mod imp {
    use super::*;
    /// Extract a tar-based archive to `out_dir` using the named tar executable.
    ///
    /// Pass `"tar"` for the system tar. Passing any name that cannot be found via
    /// `cu::which` will return an error — useful for testing error handling.
    ///
    /// The archive format is inferred from the archive extension
    ///
    /// `clean=true` wipes `out_dir` before extraction.
    #[inline(always)]
    pub fn unarchive_tar(
        tar: impl AsRef<Path>,
        archive_path: impl AsRef<Path>,
        out_dir: impl AsRef<Path>,
        clean: bool,
    ) -> cu::Result<()> {
        unarchive_tar_impl(tar.as_ref(), archive_path.as_ref(), out_dir.as_ref(), clean)
    }

    #[cu::context("failed to extract with tar: '{}'", archive_path.display())]
    fn unarchive_tar_impl(
        tar: &Path,
        archive_path: &Path,
        out_dir: &Path,
        clean: bool,
    ) -> cu::Result<()> {
        if clean {
            cu::fs::make_dir_empty(out_dir)?;
        }
        let flag = match archive_path
            .extension()
            .and_then(|e| e.to_ascii_lowercase().into_utf8().ok())
            .as_deref()
        {
            Some("gz") | Some("tgz") => "-xzf",
            Some("xz") | Some("txz") => "-xJf",
            _ => "-xf",
        };
        tar.command()
            .add(cu::args![flag, archive_path, "-C", out_dir])
            .stdout(cu::lv::D)
            .stderr(cu::lv::E)
            .stdin_null()
            .wait_nz()?;
        Ok(())
    }

    /// Extract a zip or other archive to `out_dir` using the named 7z executable.
    ///
    /// Pass `"7z"` for the system 7z. Passing any name that cannot be found via
    /// `cu::which` will return an error — useful for testing error handling.
    ///
    /// `clean=true` wipes `out_dir` before extraction.
    #[inline(always)]
    pub fn unarchive_7z(
        seven_z: impl AsRef<Path>,
        archive_path: impl AsRef<Path>,
        out_dir: impl AsRef<Path>,
        clean: bool,
    ) -> cu::Result<()> {
        unarchive_7z_impl(
            seven_z.as_ref(),
            archive_path.as_ref(),
            out_dir.as_ref(),
            clean,
        )
    }

    #[cu::context("failed to extract with 7z: '{}'", archive_path.display())]
    fn unarchive_7z_impl(
        seven_z: &Path,
        archive_path: &Path,
        out_dir: &Path,
        clean: bool,
    ) -> cu::Result<()> {
        if clean {
            cu::fs::make_dir_empty(out_dir)?;
        }
        let command = if cfg!(windows) {
            let script = format!(
                "& {} x -y {} -o{}",
                opfs::quote_path(seven_z)?,
                opfs::quote_path(archive_path)?,
                opfs::quote_path(out_dir)?,
            );
            cu::which("powershell.exe")?
                .command()
                .args(["-NoLogo", "-c", &script])
        } else {
            let out_dir_str = out_dir.as_utf8()?;
            seven_z.command().add(cu::args![
                "x",
                "-y",
                archive_path,
                format!("-o{out_dir_str}")
            ])
        };
        let (child, spinner, _) = command
            .stdoe(cu::pio::spinner("extracting").configure_spinner(|x| x.keep(false)))
            .stdin_null()
            .spawn()?;
        child.wait_nz()?;
        spinner.done();
        Ok(())
    }
}
