use std::path::{Path, PathBuf};

use corelib::hmgr;
use cu::pre::*;

use crate::init;

pub fn upgrade(path: Option<&Path>) -> cu::Result<()> {
    let temp_dir = hmgr::paths::temp_dir("core-self-upgrade");
    cu::fs::make_dir(&temp_dir)?;
    let new_binary = match path {
        Some(path) => install_from_path(path, &temp_dir)?,
        None => install_from_release(&temp_dir)?,
    };
    cu::ensure!(new_binary.exists(), "{}", new_binary.display())?;
    cu::check!(
        init::copy_new_binary(&new_binary),
        "failed to copy new binary to home"
    )?;
    cu::info!("upgrade successful - please run `shaft -vV` to run self-check and confirm");
    let _ = hmgr::paths::clean_temp_dir("core-self-upgrade");
    Ok(())
}

fn install_from_path(path: &Path, temp_dir: &Path) -> cu::Result<PathBuf> {
    let cargo = cu::check!(
        cu::which("cargo"),
        "cannot find `cargo` - cargo is required to upgrade from local path."
    )?;
    cu::info!("installing to cargo default location...");
    {
        let (child, _) = cargo
            .command()
            .current_dir(path)
            .add(cu::args!["install", "shaft-cli", "--path", "."])
            .preset(cu::pio::cargo("cargo build"))
            .spawn()?;
        cu::check!(child.wait_nz(), "failed to build new binary")?;
    }
    cu::info!("installing to home temporary location...");
    {
        let (child, _) = cargo
            .command()
            .current_dir(path)
            .add(cu::args![
                "install",
                "shaft-cli",
                "--path",
                ".",
                "--root",
                &temp_dir
            ])
            .preset(cu::pio::cargo("cargo build"))
            .spawn()?;
        cu::check!(child.wait_nz(), "failed to build new binary")?;
    }

    if cfg!(windows) {
        Ok(temp_dir.join("bin\\shaft.exe"))
    } else {
        Ok(temp_dir.join("bin/shaft"))
    }
}

fn install_from_release(temp_dir: &Path) -> cu::Result<PathBuf> {
    // TODO: cargo-binstall and fallback to cargo install --git
    // ...
    todo!()
}

pub async fn get_latest_version() -> cu::Result<String> {
    Ok(String::new())
}
