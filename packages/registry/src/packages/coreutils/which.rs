use crate::pre::*;

pub fn verify() -> cu::Result<Verified> {
    let v = check_cargo!("which" in crate "shutil-which");
    check_outdated!(&v.version, metadata[shellutils::which]::VERSION);
    Ok(Verified::UpToDate)
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    if let Ok(Verified::UpToDate) = verify() {
        return Ok(());
    }
    hmgr::repo::ensure_checkout()?;
    let crate_path = {
        let mut p = hmgr::paths::repo();
        p.extend(["packages", "shutil-which"]);
        p
    };
    epkg::cargo::install_local(&crate_path, "which", ctx.bar_ref())?;
    Ok(())
}
pub fn uninstall(_: &Context) -> cu::Result<()> {
    epkg::cargo::uninstall("shutil-which")?;
    Ok(())
}
