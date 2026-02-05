use crate::pre::*;

pub fn verify() -> cu::Result<Verified> {
    let v = check_cargo!("eza");
    check_outdated!(&v.version, metadata[coreutils::eza]::VERSION);
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::cargo::install("eza", ctx.bar_ref())
}

pub fn uninstall() -> cu::Result<()> {
    let eza_path = hmgr::paths::binary(bin_name!("eza"));
    cu::fs::remove(&eza_path)?;
    epkg::cargo::uninstall("eza")
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    // Delete existing eza binary to find the original
    let eza_path = hmgr::paths::binary(bin_name!("eza"));
    cu::fs::remove(&eza_path)?;
    let eza_src = cu::which("eza")?;
    cu::fs::copy(&eza_src, &eza_path)?;

    ctx.add_item(Item::shim_bin(
        bin_name!("ls"),
        ShimCommand::target(eza_path.into_utf8()?),
    ))?;
    Ok(())
}
