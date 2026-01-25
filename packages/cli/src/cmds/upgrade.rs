use std::path::{Path, PathBuf};

use corelib::hmgr;
use cu::pre::*;

pub fn upgrade(path: Option<&Path>) -> cu::Result<()> {
    match path {
        Some(path) => install_from_path(path)?,
        None => install_from_release()?,
    };
    cu::info!("upgrade successful - please run `shaft -vV` to run self-check and confirm");
    Ok(())
}

fn install_from_path(path: &Path) -> cu::Result<()> {
    let cargo = cu::check!(
        cu::which("cargo"),
        "cannot find `cargo` - cargo is required to upgrade from local path."
    )?;
    cu::info!("installing shaft from local path...");
    {
        let (child, _) = cargo
            .command()
            .current_dir(path)
            .add(cu::args!["install", "shaft-cli", "--path", "."])
            .preset(cu::pio::cargo("cargo build"))
            .spawn()?;
        cu::check!(child.wait_nz(), "failed to build new binary")?;
    }
    Ok(())
}

fn install_from_release() -> cu::Result<()> {
    // TODO: cargo-binstall and fallback to cargo install --git
    // ...
    todo!()
}

pub async fn get_latest_version() -> cu::Result<String> {
    Ok(String::new())
}
