use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use cu::pre::*;
use itertools::Itertools as _;

static CLEAN: AtomicBool = AtomicBool::new(false);
pub fn set_clean() {
    CLEAN.store(true, std::sync::atomic::Ordering::Release);
}

pub fn tools_dir() -> cu::Result<PathBuf> {
    Ok(packages_dir()?.join("tools"))
}

pub fn corelib_dir() -> cu::Result<PathBuf> {
    Ok(packages_dir()?.join("corelib"))
}

pub fn registry_dir() -> cu::Result<PathBuf> {
    Ok(packages_dir()?.join("registry"))
}

pub fn packages_dir() -> cu::Result<PathBuf> {
    build_crate_dir().parent_abs()
}

pub fn build_crate_dir() -> &'static Path {
    env!("CARGO_MANIFEST_DIR").as_ref()
}

pub fn write_str_if_modified(identifier: &str, path: &Path, new_content: &str) -> cu::Result<()> {
    if !CLEAN.load(std::sync::atomic::Ordering::SeqCst) {
        if let Ok(existing) = cu::fs::read_string(path) {
            if normalize_string_file_content(&existing)
                == normalize_string_file_content(new_content)
            {
                cu::hint!("not writing {identifier}: no change");
                return Ok(());
            }
        }
    }
    cu::fs::write(path, new_content)
}

pub fn write_bin_if_modified(identifier: &str, path: &Path, bytes: &[u8]) -> cu::Result<()> {
    if !CLEAN.load(std::sync::atomic::Ordering::SeqCst) {
        if let Ok(existing) = cu::fs::read(path) {
            if existing == bytes {
                cu::hint!("not writing {identifier}: no change");
                return Ok(());
            }
        }
    }
    cu::fs::write(path, bytes)
}

fn normalize_string_file_content(content: &str) -> String {
    content.lines().map(|x| x.trim()).join("\n")
}
