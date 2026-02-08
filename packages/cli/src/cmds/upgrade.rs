use std::path::Path;

use corelib::epkg;
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
        let (child, bar) = cargo
            .command()
            .current_dir(path)
            .add(cu::args!["install", "shaft-cli", "--path", "."])
            .preset(cu::pio::cargo("building shaft"))
            .spawn()?;
        cu::check!(child.wait_nz(), "failed to build new binary")?;
        bar.done();
    }
    Ok(())
}

fn install_from_release() -> cu::Result<()> {
    cu::info!("installing shaft from github...");
    let has_binstall = cu::which("cargo-binstall").is_ok();
    if !has_binstall {
        cu::hint!(
            "cargo-binstall is not installed, continuing will compile shaft from source, which will be slow."
        );
        cu::hint!("you can install cargo-binstall with `shaft sync cargo-binstall`");
        if !cu::yesno!("continue to compile from source?")? {
            cu::bail!("cancelled");
        }
    }
    let bar = cu::progress("installing shaft").spawn();
    if has_binstall {
        epkg::cargo::binstall_git(
            "shaft-cli",
            "https://github.com/Pistonite/shaft",
            Some(&bar),
        )?;
    } else {
        epkg::cargo::install_git_commit(
            "shaft-cli",
            "https://github.com/Pistonite/shaft",
            "main",
            Some(&bar),
        )?;
    }
    bar.done();
    Ok(())
}
