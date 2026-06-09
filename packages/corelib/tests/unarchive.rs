use std::path::{Path, PathBuf};

use cu::pre::*;
use shaft_corelib::opfs;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn make_test_dir(name: &str) -> cu::Result<PathBuf> {
    let temp_name = format!("shaft-corelib-test-{name}");
    let dir = cu::path!((std::env::temp_dir()) / temp_name);
    cu::fs::make_dir_empty(&dir)?;
    Ok(dir)
}

/// Copy fixture files into `<dir>/testpkg/` for archive creation.
fn populate_testpkg(dir: &Path) -> cu::Result<()> {
    let fixtures = fixtures_dir();
    let testpkg = dir.join("testpkg");
    cu::fs::make_dir(testpkg.join("subdir"))?;
    cu::fs::copy(fixtures.join("a.txt"), testpkg.join("a.txt"))?;
    cu::fs::copy(fixtures.join("subdir/b.txt"), testpkg.join("subdir/b.txt"))?;
    Ok(())
}

fn assert_extracted(extract_dir: &Path) -> cu::Result<()> {
    let fixtures = fixtures_dir();
    let a = extract_dir.join("testpkg/a.txt");
    let b = extract_dir.join("testpkg/subdir/b.txt");
    assert!(a.exists(), "missing: {}", a.display());
    assert!(b.exists(), "missing: {}", b.display());
    assert_eq!(
        cu::fs::read_string(&a)?,
        cu::fs::read_string(fixtures.join("a.txt"))?,
        "a.txt content mismatch"
    );
    assert_eq!(
        cu::fs::read_string(&b)?,
        cu::fs::read_string(fixtures.join("subdir/b.txt"))?,
        "subdir/b.txt content mismatch"
    );
    Ok(())
}

#[test]
fn test_tar_not_found() -> cu::Result<()> {
    let tmp = make_test_dir("tar_not_found")?;
    let result = opfs::imp::unarchive_tar(
        "does-not-exist-tar",
        "/nonexistent/archive.tar.gz",
        &tmp,
        false,
    );
    assert!(result.is_err(), "expected error when tar not found");
    Ok(())
}

#[test]
fn test_unarchive_tar() -> cu::Result<()> {
    let tmp = make_test_dir("tar")?;
    populate_testpkg(&tmp)?;

    let archive = tmp.join("test.tar");
    cu::which("tar")?
        .command()
        .add(cu::args!["-cf", &archive, "-C", &tmp, "testpkg"])
        .all_null()
        .wait_nz()?;

    let extract_dir = tmp.join("extract");
    cu::fs::make_dir(&extract_dir)?;
    opfs::unarchive(&archive, &extract_dir, false)?;
    assert_extracted(&extract_dir)?;

    let _ = cu::fs::rec_remove(&tmp);
    Ok(())
}

#[test]
fn test_unarchive_tar_gz() -> cu::Result<()> {
    let tmp = make_test_dir("tar_gz")?;
    populate_testpkg(&tmp)?;

    let archive = tmp.join("test.tar.gz");
    cu::which("tar")?
        .command()
        .add(cu::args!["-czf", &archive, "-C", &tmp, "testpkg"])
        .all_null()
        .wait_nz()?;

    let extract_dir = tmp.join("extract");
    cu::fs::make_dir(&extract_dir)?;
    opfs::unarchive(&archive, &extract_dir, false)?;
    assert_extracted(&extract_dir)?;

    let _ = cu::fs::rec_remove(&tmp);
    Ok(())
}

#[test]
fn test_unarchive_tar_xz() -> cu::Result<()> {
    let tmp = make_test_dir("tar_xz")?;
    populate_testpkg(&tmp)?;

    let archive = tmp.join("test.tar.xz");
    cu::which("tar")?
        .command()
        .add(cu::args!["-cJf", &archive, "-C", &tmp, "testpkg"])
        .all_null()
        .wait_nz()?;

    let extract_dir = tmp.join("extract");
    cu::fs::make_dir(&extract_dir)?;
    opfs::unarchive(&archive, &extract_dir, false)?;
    assert_extracted(&extract_dir)?;

    let _ = cu::fs::rec_remove(&tmp);
    Ok(())
}

#[test]
fn test_7z_not_found() -> cu::Result<()> {
    let tmp = make_test_dir("7z_not_found")?;
    let result =
        opfs::imp::unarchive_7z("does-not-exist-7z", "/nonexistent/archive.zip", &tmp, false);
    assert!(result.is_err(), "expected error when 7z not found");
    Ok(())
}

#[test]
fn test_unarchive_zip() -> cu::Result<()> {
    let tmp = make_test_dir("zip")?;
    populate_testpkg(&tmp)?;

    let archive = tmp.join("test.zip");
    cu::which("7z")?
        .command()
        .add(cu::args!["a", "-tzip", &archive, "testpkg"])
        .current_dir(&tmp)
        .all_null()
        .wait_nz()?;

    let extract_dir = tmp.join("extract");
    cu::fs::make_dir(&extract_dir)?;
    opfs::unarchive(&archive, &extract_dir, false)?;
    assert_extracted(&extract_dir)?;

    let _ = cu::fs::rec_remove(&tmp);
    Ok(())
}

#[test]
fn test_unarchive_rename() -> cu::Result<()> {
    let tmp = make_test_dir("rename")?;
    populate_testpkg(&tmp)?;

    let archive = tmp.join("test.tar.gz");
    cu::which("tar")?
        .command()
        .add(cu::args!["-czf", &archive, "-C", &tmp, "testpkg"])
        .all_null()
        .wait_nz()?;

    let extract_dir = tmp.join("extract");
    cu::fs::make_dir(&extract_dir)?;
    opfs::unarchive_rename(
        &archive,
        &extract_dir,
        extract_dir.join("testpkg"),
        extract_dir.join("renamed"),
        false,
        None,
    )?;

    let renamed_a = extract_dir.join("renamed/a.txt");
    let renamed_b = extract_dir.join("renamed/subdir/b.txt");
    assert!(renamed_a.exists(), "missing: {}", renamed_a.display());
    assert!(renamed_b.exists(), "missing: {}", renamed_b.display());
    assert!(
        !extract_dir.join("testpkg").exists(),
        "testpkg should be gone after rename"
    );
    assert_eq!(
        cu::fs::read_string(&renamed_a)?,
        cu::fs::read_string(fixtures_dir().join("a.txt"))?,
        "a.txt content mismatch"
    );

    let _ = cu::fs::rec_remove(&tmp);
    Ok(())
}
