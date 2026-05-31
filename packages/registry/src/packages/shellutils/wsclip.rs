use crate::pre::*;

pub fn verify() -> cu::Result<Verified> {
    let v = check_cargo!("wsclip" in crate "shutil-wsclip");
    check_outdated!(&v.version, metadata[shellutils::wsclip]::VERSION);
    Ok(Verified::UpToDate)
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    if let Ok(Verified::UpToDate) = verify() {
        return Ok(());
    }
    hmgr::repo::ensure_checkout()?;
    let crate_path = {
        let mut p = hmgr::paths::repo();
        p.extend(["packages", "shutil-wsclip"]);
        p
    };
    epkg::cargo::install_local(&crate_path, "wsclip", ctx.bar_ref())?;
    Ok(())
}
pub fn uninstall(_: &Context) -> cu::Result<()> {
    epkg::cargo::uninstall("shutil-wsclip")?;
    Ok(())
}
