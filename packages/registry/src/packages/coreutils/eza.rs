use crate::pre::*;

pub fn verify() -> cu::Result<Verified> {
    check_in_shaft!("eza");
    let version = get_version()?;
    check_outdated!(&version, metadata[coreutils::eza]::VERSION);
    Ok(Verified::UpToDate)
}

fn get_version() -> cu::Result<String> {
    let output = command_output!("eza", ["--version"]);
    let output = cu::check!(
        output.lines().find(|l| l.starts_with("v")),
        "failed to parse eza --version output: failed to find version line"
    )?;
    let version = output.strip_prefix('v').unwrap_or(output);
    let version = version.split_once(' ').map(|a| a.0).unwrap_or(version);
    Ok(version.trim().to_string())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    if let Ok(Verified::UpToDate) = verify() {
        return Ok(());
    }
    let install_dir = cu::path!((ctx.install_dir()) / "eza").into_utf8()?;
    epkg::cargo::install("eza", Some(&install_dir), ctx.bar_ref())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let bin = cu::path!((ctx.install_dir()) / "eza" / "bin" / bin_name!("eza")).into_utf8()?;
    ctx.add_item(Item::link_bin(bin_name!("ls"), bin.clone()))?;
    ctx.add_item(Item::link_bin(bin_name!("eza"), bin))?;
    Ok(())
}
