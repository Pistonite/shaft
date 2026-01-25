use crate::pre::*;

pub fn verify() -> cu::Result<Verified> {
    let info = check_installed_with_cargo!("eza");
    let is_not_uptodate = Version(&info.version) < metadata::coreutils::eza::VERSION;
    Ok(Verified::is_uptodate(!is_not_uptodate))
}

pub fn install() -> cu::Result<()> {
    epkg::cargo::install("eza")
}

pub fn uninstall() -> cu::Result<()> {
    epkg::cargo::uninstall("eza")
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    // Delete existing eza binary to find the original
    let eza_path = hmgr::paths::binary(bin_name!("eza"));
    cu::fs::remove(&eza_path)?;
    let eza_src = cu::which("eza")?;
    cu::fs::copy(&eza_src, &eza_path)?;

    ctx.add_item(hmgr::Item::ShimBin(bin_name!("ls").to_string(),
        vec![eza_path.into_utf8()?]
    ))?;
    Ok(())
}
