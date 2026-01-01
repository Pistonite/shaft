use std::path::Path;

use cu::pre::*;

pub fn check_init_binary() -> cu::Result<()> {
    let current_exe = cu::fs::current_exe()?;
    let home_binary = op::home::shaft_binary();

    if home_binary != current_exe {
        cu::warn!("current binary is not located in home");
        cu::hint!("this is expected when initializing the tool for the first time");
        copy_new_binary(&current_exe)?;
        // if PATH is configured correctly, then the result from 'which'
        // should be the new binary
        if let Err(e) = check_binary_location(&home_binary) {
            cu::error!("{e:?}");
            cu::warn!(
                "binary location check failed - please ensure your PATH is configured correctly; restarting the terminal/shell is recommended"
            );
            cu::bail!("home binary updated, please check your PATH and rerun");
        } else {
            cu::bail!("home binary updated, please rerun")
        }
    }

    if let Err(e) = check_binary_location(&home_binary) {
        cu::warn!(
            "binary location check failed - please ensure your PATH is configured correctly; restarting the terminal/shell is recommended"
        );
        cu::rethrow!(e, "binary location check failed");
    }
    let home_binary_old = op::home::shaft_binary_old();
    let _ = cu::fs::remove(&home_binary_old);

    Ok(())
}

pub fn copy_new_binary(new_binary: &Path) -> cu::Result<()> {
    // rename old binary
    let home_binary = op::home::shaft_binary();
    let home_binary_old = op::home::shaft_binary_old();
    cu::check!(
        cu::fs::remove(&home_binary_old),
        "failed to remove old old binary"
    )?;
    if home_binary.exists() {
        cu::check!(
            cu::fs::copy(&home_binary, &home_binary_old),
            "failed to copy old binary"
        )?;
        cu::check!(cu::fs::remove(&home_binary), "failed to remove old binary")?;
    }
    cu::check!(
        cu::fs::copy(new_binary, &home_binary),
        "failed to copy new binary to home"
    )?;
    cu::info!("successfully copied binary to home");
    Ok(())
}

fn check_binary_location(home_binary: &Path) -> cu::Result<()> {
    let binary_in_path = cu::which("shaft")?;
    if home_binary != &binary_in_path {
        cu::bail!(
            "binary in PATH is not the same as home: '{}' (home is '{}')",
            binary_in_path.display(),
            home_binary.display()
        );
    }
    Ok(())
}

pub fn upgrade_binary(path: Option<&Path>) -> cu::Result<()> {
    let temp_dir = op::home::temp_dir("core-self-upgrade");
    cu::fs::make_dir(&temp_dir)?;
    let new_binary = match path {
        Some(path) => {
            let cargo = cu::check!(
                cu::which("cargo"),
                "cannot find `cargo` - cargo is required to upgrade from local path."
            )?;
            cu::info!("installing to cargo default location...");
            {
                let (child, _progress, _progress2) = cargo
                    .command()
                    .current_dir(path)
                    .add(cu::args!["install", "shaft-cli", "--path", "."])
                    .preset(cu::pio::cargo())
                    .spawn()?;
                cu::check!(child.wait_nz(), "failed to build new binary")?;
            }
            cu::info!("installing to home temporary location...");
            {
                let (child, _progress, _progress2) = cargo
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
                    .preset(cu::pio::cargo())
                    .spawn()?;
                cu::check!(child.wait_nz(), "failed to build new binary")?;
            }

            if cfg!(windows) {
                temp_dir.join("bin\\shaft.exe")
            } else {
                temp_dir.join("bin/shaft")
            }
        }
        None => {
            // TODO: cargo-binstall and fallback to cargo install --git
            // ...
            todo!()
        }
    };
    cu::ensure!(new_binary.exists(), "failed to locate new binary");
    cu::check!(
        copy_new_binary(&new_binary),
        "failed to copy new binary to home"
    )?;
    cu::info!("upgrade successful - please run `shaft -vV` to run self-check and confirm");
    let _ = op::home::clean_temp_dir("core-self-upgrade");
    Ok(())
}
