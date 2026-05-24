use crate::pre::*;

pub fn verify() -> cu::Result<Verified> {
    let v = check_cargo!("viopen" in crate "shutil-viopen");
    check_outdated!(&v.version, metadata[shellutils::viopen]::VERSION);
    Ok(Verified::UpToDate)
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    if let Ok(Verified::UpToDate) = verify() {
        return Ok(());
    }
    hmgr::repo::ensure_checkout()?;
    let crate_path = {
        let mut p = hmgr::paths::repo();
        p.extend(["packages", "shutil-viopen"]);
        p
    };
    epkg::cargo::install_local(&crate_path, "viopen", ctx.bar_ref())?;
    Ok(())
}
pub fn uninstall(_: &Context) -> cu::Result<()> {
    epkg::cargo::uninstall("shutil-viopen")?;
    Ok(())
}
