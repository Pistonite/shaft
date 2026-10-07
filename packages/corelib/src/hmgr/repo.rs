use std::sync::LazyLock;

use cu::pre::*;

use crate::{bin_name, epkg, hmgr, opfs};

static SHAFT_REPO: &str = "https://github.com/Pistonite/shaft";

/// Get the current commit of the local repo (if checked out)
pub fn get_commit() -> cu::Result<Option<String>> {
    let repo_path = hmgr::paths::repo();

    if !repo_path.exists() {
        return Ok(None);
    }

    let (child, output) = cu::which("git")?
        .command()
        .current_dir(&repo_path)
        .args(["rev-parse", "HEAD"])
        .stdout(cu::pio::string())
        .stderr(cu::lv::P)
        .stdin_null()
        .spawn()?;
    child.wait_nz()?;
    let output = output.join()??;

    Ok(Some(output.trim().to_string()))
}

static CHECKOUT: LazyLock<cu::Result<()>> = LazyLock::new(|| {
    let repo_path = hmgr::paths::repo();

    if !repo_path.exists() {
        cu::fs::make_dir(&repo_path)?;
        cu::which("git")?
            .command()
            .add(cu::args!["clone", SHAFT_REPO, &repo_path])
            .stdout(cu::lv::P)
            .stderr(cu::lv::P)
            .stdin_null()
            .wait_nz()?;
    }

    cu::which("git")?
        .command()
        .current_dir(&repo_path)
        .args(["fetch", "origin", "main"])
        .stdout(cu::lv::P)
        .stderr(cu::lv::P)
        .stdin_null()
        .wait_nz()?;

    cu::which("git")?
        .command()
        .current_dir(&repo_path)
        .args(["reset", "--hard", "origin/main"])
        .stdout(cu::lv::P)
        .stderr(cu::lv::P)
        .stdin_null()
        .wait_nz()?;
    cu::Ok(())
});

pub fn ensure_checkout() -> cu::Result<()> {
    match &*CHECKOUT {
        Err(e) => {
            cu::error!("repo checkout error: {e:?}");
            cu::bail!("failed to checkout shaft repo");
        }
        Ok(()) => {
            let repo_path = hmgr::paths::repo();
            if !repo_path.exists() {
                cu::bail!("cannot find shaft repo, please try again");
            }
            Ok(())
        }
    }
}

/// Build shaft from source locally and update the current executable
pub fn local_update() -> cu::Result<()> {
    let repo_path = hmgr::paths::repo();
    ensure_checkout()?;

    let current_version = opfs::cli_version();
    let next_version = match get_cli_version() {
        Err(e) => {
            cu::warn!("failed to get next CLI version! {e:?}");
            None
        }
        Ok(None) => {
            cu::warn!("failed to get next CLI version!");
            None
        }
        Ok(Some(v)) => {
            cu::hint!("upgrading: {current_version} -> {v}");
            Some(v)
        }
    };

    // the build script is cached and only rebuilt if minor version is bumped
    // (for performance reasons)
    let build_script = repo_path.join(".cache").join(bin_name!("shaft-build"));
    let use_cached_build_script = build_script.exists()
        && next_version.is_some_and(|v| is_same_minor_version(current_version, &v));
    if use_cached_build_script {
        cu::hint!("using cached pre-build script");
    } else {
        let command = cu::which("cargo")?.command().current_dir(&repo_path).args([
            "build",
            "--bin",
            "shaft-build",
            "--locked",
        ]);
        let command = epkg::cargo::add_platform_build_args(command);
        let (child, bar) = command
            .preset(cu::pio::cargo("building pre-build script"))
            .spawn()?;
        child.wait_nz()?;
        bar.done();

        #[cfg(feature = "build-x64")]
        let build_output = repo_path
            .join("target")
            .join(epkg::cargo::BUILD_X64_TARGET_TRIPLE)
            .join("debug")
            .join(bin_name!("shaft-build"));
        #[cfg(not(feature = "build-x64"))]
        let build_output = repo_path
            .join("target")
            .join("debug")
            .join(bin_name!("shaft-build"));
        cu::check!(
            cu::fs::copy(build_output, &build_script),
            "failed to cache pre-build script"
        )?;
    }
    {
        build_script
            .command()
            .current_dir(&repo_path)
            .all_inherit()
            .wait_nz()?;
    }
    {
        let command = cu::which("cargo")?.command().current_dir(&repo_path).args([
            "build",
            "--bin",
            "shaft",
            "--release",
            "--locked",
        ]);
        #[cfg(feature = "build-x64")]
        let command = command.args(["--features", "build-x64"]);
        let command = epkg::cargo::add_platform_build_args(command);
        let (child, bar) = command.preset(cu::pio::cargo("building")).spawn()?;
        child.wait_nz()?;
        bar.done();
    }

    #[cfg(feature = "build-x64")]
    let output_path = repo_path
        .join("target")
        .join(epkg::cargo::BUILD_X64_TARGET_TRIPLE)
        .join("release")
        .join(bin_name!("shaft"));
    #[cfg(not(feature = "build-x64"))]
    let output_path = repo_path
        .join("target")
        .join("release")
        .join(bin_name!("shaft"));
    let expected_path = hmgr::paths::binary(bin_name!("shaft"));

    let current_exe = std::env::current_exe()?;
    let exe_old = current_exe.with_extension(if cfg!(windows) { "old.exe" } else { "old" });
    if exe_old.exists() {
        cu::check!(
            cu::fs::remove(&exe_old),
            "failed to remove old executable at '{}'",
            exe_old.display()
        )?;
    }
    let current_exe_norm = current_exe.normalize()?.to_string_lossy().to_lowercase();
    let expected_exe_norm = expected_path.to_string_lossy().to_lowercase();
    let is_current_exe_in_shaft = current_exe_norm == expected_exe_norm;
    if is_current_exe_in_shaft {
        std::fs::rename(&current_exe, &exe_old)?;
    }
    cu::check!(
        cu::fs::copy(output_path, &expected_path),
        "failed to copy build output to bin"
    )?;
    cu::info!("copied build output to $SHAFT_HOME/bin");
    Ok(())
}

/// Check if 2 versions have the same major and minor version
fn is_same_minor_version(a: &str, b: &str) -> bool {
    fn major_minor(v: &str) -> (Option<&str>, Option<&str>) {
        let mut parts = v.trim().split('.');
        (parts.next(), parts.next())
    }
    major_minor(a) == major_minor(b)
}

/// Get the current version of the CLI as specified in the repo (not the version
/// of the running binary)
pub fn get_cli_version() -> cu::Result<Option<String>> {
    let path = cu::path!((hmgr::paths::repo()) / "VERSION");

    if !path.exists() {
        return Ok(None);
    }

    let v = cu::fs::read_string(path)?.trim().to_string();
    Ok(Some(v))
}
