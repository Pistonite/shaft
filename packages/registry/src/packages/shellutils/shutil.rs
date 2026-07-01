use crate::pre::*;

pub fn verify() -> cu::Result<Verified> {
    check_verified!(verify_n()?);
    check_verified!(verify_viopen()?);
    check_verified!(verify_lfmt()?);
    if cfg!(windows) {
        check_verified!(verify_vipath()?);
        check_verified!(verify_wsclip()?);
    }
    Ok(Verified::UpToDate)
}

pub fn verify_n() -> cu::Result<Verified> {
    let v = check_cargo!("n" in crate "shutil-n");
    check_outdated!(&v.version, metadata[shellutils::n]::VERSION);
    Ok(Verified::UpToDate)
}
pub fn verify_viopen() -> cu::Result<Verified> {
    let v = check_cargo!("viopen" in crate "shutil-viopen");
    check_outdated!(&v.version, metadata[shellutils::viopen]::VERSION);
    Ok(Verified::UpToDate)
}
pub fn verify_lfmt() -> cu::Result<Verified> {
    let v = check_cargo!("lfmt" in crate "shutil-lfmt");
    check_outdated!(&v.version, metadata[shellutils::lfmt]::VERSION);
    Ok(Verified::UpToDate)
}
pub fn verify_vipath() -> cu::Result<Verified> {
    let v = check_cargo!("vipath" in crate "shutil-vipath");
    check_outdated!(&v.version, metadata[shellutils::vipath]::VERSION);
    Ok(Verified::UpToDate)
}
pub fn verify_wsclip() -> cu::Result<Verified> {
    let v = check_cargo!("wsclip" in crate "shutil-wsclip");
    check_outdated!(&v.version, metadata[shellutils::wsclip]::VERSION);
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    let mut need_install = false;
    let need_n = !matches!(verify_n(), Ok(Verified::UpToDate));
    need_install = need_install || need_n;
    let need_viopen = !matches!(verify_viopen(), Ok(Verified::UpToDate));
    need_install = need_install || need_viopen;
    let need_lfmt = !matches!(verify_lfmt(), Ok(Verified::UpToDate));
    need_install = need_install || need_lfmt;
    let need_vipath = cfg!(windows) && !matches!(verify_vipath(), Ok(Verified::UpToDate));
    need_install = need_install || need_vipath;
    let need_wsclip = cfg!(windows) && !matches!(verify_wsclip(), Ok(Verified::UpToDate));
    need_install = need_install || need_wsclip;

    if !need_install {
        return Ok(());
    }

    hmgr::repo::ensure_checkout()?;

    let packages_path = hmgr::paths::repo().join("packages");
    if need_n {
        epkg::cargo::install_local(&packages_path.join("shutil-n"), "n", ctx.bar_ref())?;
    }
    if need_viopen {
        epkg::cargo::install_local(&packages_path.join("shutil-viopen"), "n", ctx.bar_ref())?;
    }
    if need_lfmt {
        epkg::cargo::install_local(&packages_path.join("shutil-lfmt"), "n", ctx.bar_ref())?;
    }
    if need_vipath {
        epkg::cargo::install_local(&packages_path.join("shutil-vipath"), "n", ctx.bar_ref())?;
    }
    if need_wsclip {
        epkg::cargo::install_local(&packages_path.join("shutil-wsclip"), "n", ctx.bar_ref())?;
    }
    Ok(())
}
pub fn uninstall(_: &Context) -> cu::Result<()> {
    epkg::cargo::uninstall("shutil-n")?;
    epkg::cargo::uninstall("shutil-viopen")?;
    epkg::cargo::uninstall("shutil-lfmt")?;
    if cfg!(windows) {
        epkg::cargo::uninstall("shutil-vipath")?;
        epkg::cargo::uninstall("shutil-wsclip")?;
    }
    Ok(())
}
