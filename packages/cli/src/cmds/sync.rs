use cu::pre::*;
use enumset::EnumSet;
use registry::{Context, PkgId, Verified};

use crate::graph::{self, InstallCache};

pub fn sync(packages: &[String]) -> cu::Result<()> {
    let pkgs = graph::parse_pkgs(packages)?;
    let mut installed = InstallCache::load()?;
    let pkgs = if pkgs.is_empty() {
        // sync all installed packages
        installed.pkgs
    } else {
        pkgs
    };
    sync_pkgs(pkgs, &mut installed)
}

pub fn sync_pkgs(pkgs: EnumSet<PkgId>, installed: &mut InstallCache) -> cu::Result<()> {
    if pkgs.is_empty() {
        return Ok(());
    }
    let graph = graph::build_sync_graph(pkgs, &installed, &mut Default::default())?;
    match graph.len() {
        1 => cu::info!("syncing 1 package..."),
        x => cu::info!("syncing {x} packages..."),
    }
    for pkg in graph {
        cu::check!(do_sync_package(pkg), "failed to sync '{pkg}'")?;
        installed.add(pkg)?;
        installed.save()?;
    }
    Ok(())
}

fn do_sync_package(pkg: PkgId) -> cu::Result<()> {
    let package = pkg.package();
    let ctx = Context { pkg };
    match package.verify(&ctx)? {
        Verified::UpToDate => {
            // TODO: check config dirty
            cu::info!("up to date: '{pkg}'");
            return Ok(());
        }
        Verified::NotUpToDate => {
            // TODO: backup
            cu::debug!("needs update: '{pkg}'");
        }
        Verified::NotInstalled => {}
    }
    let bar = cu::progress_bar(5, format!("sync '{pkg}'"));

    cu::progress!(&bar, 0, "downloading");
    package.download(&ctx)?;
    cu::progress!(&bar, 1, "building");
    package.build(&ctx)?;
    cu::progress!(&bar, 2, "installing");
    package.install(&ctx)?;
    cu::progress!(&bar, 3, "configuring");
    package.configure(&ctx)?;
    cu::progress!(&bar, 4, "cleaning");
    package.clean(&ctx)?;

    match package.verify(&ctx)? {
        Verified::UpToDate => {
            cu::progress_done!(&bar, "synced '{pkg}'");
        }
        _ => {
            cu::bail!("verification failed after installation");
        }
    }

    Ok(())
}
