use cu::pre::*;

/// Install a winget package
#[cu::context("failed to install {id} with winget")]
pub fn install(id: &str) -> cu::Result<()> {
    cu::info!("installing {id} with winget");
    cu::which("winget")?
        .command()
        .args(["install", id])
        .stdoe(cu::lv::D)
        .stdin_null()
        .wait_nz()?;
    cu::info!("installed {id} with winget");
    Ok(())
}

/// Uninstall a winget package
#[cu::context("failed to uninstall {id} with winget")]
pub fn uninstall(id: &str) -> cu::Result<()> {
    cu::info!("uninstalling {id} with winget");
    cu::which("winget")?
        .command()
        .args(["uninstall", id])
        .stdoe(cu::lv::D)
        .stdin_null()
        .wait_nz()?;
    cu::info!("uninstalled {id} with winget");
    Ok(())
}
