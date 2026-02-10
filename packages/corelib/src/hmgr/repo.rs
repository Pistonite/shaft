use cu::pre::*;

use crate::{bin_name, hmgr};

static SHAFT_REPO: &str = "https://github.com/Pistonite/shaft";

/// Build shaft from source locally and update the current executable
pub fn local_update() -> cu::Result<()> {
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
        .add(cu::args!["fetch", "origin", "main"])
        .stdout(cu::lv::P)
        .stderr(cu::lv::P)
        .stdin_null()
        .wait_nz()?;

    cu::which("git")?
        .command()
        .current_dir(&repo_path)
        .add(cu::args!["reset", "--hard", "origin/main"])
        .stdout(cu::lv::P)
        .stderr(cu::lv::P)
        .stdin_null()
        .wait_nz()?;

    {
        let (child, bar) = cu::which("cargo")?
            .command()
            .current_dir(&repo_path)
            .add(cu::args!["build", "--bin", "shaft-build", "--locked"])
            .preset(cu::pio::cargo("running pre-build script"))
            .spawn()?;
        child.wait_nz()?;
        bar.done();
    }
    {
        let (child, bar) = cu::which("cargo")?
            .command()
            .current_dir(&repo_path)
            .add(cu::args![
                "build",
                "--bin",
                "shaft",
                "--release",
                "--locked"
            ])
            .preset(cu::pio::cargo("building"))
            .spawn()?;
        child.wait_nz()?;
        bar.done();
    }

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
    if !is_current_exe_in_shaft {
        cu::hint!("you should remove the existing installation (e.g cargo uninstall shaft-cli)");
    } else {
        cu::info!("update successful");
    }
    Ok(())
}
