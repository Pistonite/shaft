use std::path::PathBuf;

use cu::pre::*;
use registry::{Context, PkgId, Stage};

use crate::graph::InstallCache;


pub fn config(package: &str) -> cu::Result<()> {
    let pkg = cu::check!(PkgId::from_str(package), "cannot find package '{package}'")?;
    let config_location = config_location_path(pkg)?;
    let mut installed = InstallCache::load()?;
    installed.set_dirty(pkg, true);
    cu::check!(installed.save(), "failed to mark configuration for '{pkg}' as dirty")?;
    if config_location.is_dir() {
        cu::hint!(r"the config location for '{pkg}' is a directory.
you can print the path with

    shaft config -l {pkg}

the config for this package has been marked dirty;
after editing the config, please run `shaft sync`
");
        cu::bail!("please print the config location and manually edit it");
    }
    let content = cu::fs::read_string(&config_location).ok();
    viopen::open(&config_location)?;
    let content_after = cu::fs::read_string(&config_location).ok();
    if content.is_some() && content == content_after {
        cu::info!("no change");
        return Ok(());
    }
    cu::hint!(r"configuration saved; after done configuring, please run `shaft sync`");
    Ok(())
}

pub fn config_location(package: &str) -> cu::Result<String> {
    let pkg = cu::check!(PkgId::from_str(package), "cannot find package '{package}'")?;
    Ok(config_location_path(pkg)?.into_utf8()?)
}

pub fn config_location_path(pkg: PkgId) -> cu::Result<PathBuf> {
    let mut ctx = Context::new(Default::default());
    ctx.pkg = pkg;
    ctx.stage.set(Stage::Configure);
    let location = pkg.package().config_location(&ctx)?;
    cu::check!(location, "package '{pkg}' does not have a config file")
}
