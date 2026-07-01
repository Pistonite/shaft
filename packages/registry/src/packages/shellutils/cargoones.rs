use crate::pre::*;

pub fn verify() -> cu::Result<Verified> {
    let v = check_cargo!("bat");
    check_outdated!(&v.version, metadata[bat]::VERSION);
    let v = check_cargo!("dust" in crate "du-dust");
    check_outdated!(&v.version, metadata[dust]::VERSION);
    let v = check_cargo!("fd" in crate "fd-find");
    check_outdated!(&v.version, metadata[fd]::VERSION);
    let v = check_cargo!("rg" in crate "ripgrep");
    check_outdated!(&v.version, metadata[rg]::VERSION);
    let v = check_cargo!("websocat");
    check_outdated!(&v.version, metadata[websocat]::VERSION);
    let v = check_cargo!("zoxide");
    check_outdated!(&v.version, metadata[zoxide]::VERSION);
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::cargo::binstall("bat", ctx.bar_ref())?;
    epkg::cargo::binstall("du-dust", ctx.bar_ref())?;
    epkg::cargo::install("fd-find", None, ctx.bar_ref())?;
    epkg::cargo::binstall("ripgrep", ctx.bar_ref())?;
    epkg::cargo::install("websocat", None, ctx.bar_ref())?;
    epkg::cargo::install("zoxide", None, ctx.bar_ref())?;
    Ok(())
}
pub fn uninstall(_: &Context) -> cu::Result<()> {
    epkg::cargo::uninstall("bat")?;
    epkg::cargo::uninstall("du-dust")?;
    epkg::cargo::uninstall("fd-find")?;
    epkg::cargo::uninstall("ripgrep")?;
    epkg::cargo::uninstall("websocat")?;
    epkg::cargo::uninstall("zoxide")?;
    Ok(())
}
