use crate::pre::*;

pub fn verify() -> cu::Result<Verified> {
    check_in_shaft!("fzf");
    // "0.74.5 (abcdef)"
    let v = command_output!("fzf", ["--version"]);
    let v = v.split_once(' ').map(|x| x.0).unwrap_or(&v);
    check_outdated!(v.trim(), metadata[fzf]::VERSION);
    Ok(Verified::UpToDate)
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file(
        fzf_filename(),
        metadata::fzf::URL(),
        metadata::fzf::SHA(),
        ctx.bar(),
    )?;
    Ok(())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    let install_dir = ctx.install_dir();
    cu::fs::make_dir(&install_dir)?;

    // the archives only contain the executable, without a top-level directory
    let archive = hmgr::paths::download(fzf_filename(), metadata::fzf::URL());
    opfs::unarchive(archive, &install_dir, false)?;
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    ctx.add_item(Item::link_bin(
        hmgr::paths::binary(bin_name!("fzf")).into_utf8()?,
        ctx.install_dir().join(bin_name!("fzf")).into_utf8()?,
    ))?;
    Ok(())
}

fn fzf_filename() -> &'static str {
    if cfg!(windows) { "fzf.zip" } else { "fzf.tgz" }
}
