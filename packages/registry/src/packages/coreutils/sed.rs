use crate::pre::*;

pub fn verify() -> cu::Result<Verified> {
    let v = check_cargo!("sed");
    check_outdated!(&v.version, metadata[coreutils::uutils_sed]::VERSION);
    Ok(Verified::UpToDate)
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    if let Ok(Verified::UpToDate) = verify() {
        return Ok(());
    }
    epkg::cargo::binstall("sed", ctx.bar_ref())?;
    Ok(())
}
pub fn uninstall(_: &Context) -> cu::Result<()> {
    epkg::cargo::uninstall("sed")?;
    Ok(())
}
