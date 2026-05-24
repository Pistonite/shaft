use crate::pre::*;

pub fn verify() -> cu::Result<Verified> {
    let v = check_cargo!("lfmt" in crate "shutil-lfmt");
    check_outdated!(&v.version, metadata[shellutils::lfmt]::VERSION);
    Ok(Verified::UpToDate)
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    if let Ok(Verified::UpToDate) = verify() {
        return Ok(());
    }
    hmgr::repo::ensure_checkout()?;
    let crate_path = {
        let mut p = hmgr::paths::repo();
        p.extend(["packages", "shutil-lfmt"]);
        p
    };
    epkg::cargo::install_local(&crate_path, "lfmt", ctx.bar_ref())?;
    Ok(())
}
pub fn uninstall(_: &Context) -> cu::Result<()> {
    epkg::cargo::uninstall("shutil-lfmt")?;
    Ok(())
}
