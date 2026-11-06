use crate::pre::*;

pub fn verify() -> cu::Result<Verified> {
    check_in_shaft!("jq");
    // "jq-1.8.2"
    let v = command_output!("jq", ["--version"]);
    let v = v.strip_prefix("jq-").unwrap_or(&v);
    check_outdated!(v.trim(), metadata[jq]::VERSION);
    Ok(Verified::UpToDate)
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file(
        bin_name!("jq"),
        metadata::jq::URL(),
        metadata::jq::SHA(),
        ctx.bar(),
    )?;
    Ok(())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    let install_dir = ctx.install_dir();
    cu::fs::make_dir(&install_dir)?;

    // the release assets are the executable itself, not an archive
    let jq_download = hmgr::paths::download(bin_name!("jq"), metadata::jq::URL());
    let jq_target = install_dir.join(bin_name!("jq"));
    cu::fs::copy(jq_download, &jq_target)?;
    opfs::set_executable(&jq_target)?;
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    ctx.add_item(Item::link_bin(
        hmgr::paths::binary(bin_name!("jq")).into_utf8()?,
        ctx.install_dir().join(bin_name!("jq")).into_utf8()?,
    ))?;
    Ok(())
}
