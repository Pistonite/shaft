use corelib::ItemMgr;
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
    let items = ItemMgr::load()?;
    let mut ctx = Context::new(items);
    for pkg in graph {
        ctx.pkg = pkg;
        ctx = cu::check!(do_sync_package(ctx), "failed to sync '{pkg}'")?;
        ctx.set_bar(None);
        installed.add(pkg)?;
        installed.save()?;
    }
    Ok(())
}

fn do_sync_package(mut ctx: Context) -> cu::Result<Context> {
    let pkg = ctx.pkg;
    let package = ctx.pkg.package();
    let needs_backup = match package.verify(&ctx)? {
        Verified::UpToDate => {
            // TODO: check config dirty
            cu::info!("up to date: '{pkg}'");
            return Ok(ctx);
        }
        Verified::NotUpToDate => {
            cu::debug!("needs update: '{pkg}'");
            true
        }
        Verified::NotInstalled => false,
    };
    let bar = cu::progress(format!("sync '{pkg}'")).spawn();
    ctx.set_bar(Some(&bar));
    let mut backup_guard = if needs_backup {
        cu::progress!(bar, "backup");
        Some(package.backup_guard(&ctx)?)
    } else {
        None
    };

    cu::progress!(bar, "downloading");
    package.download(&ctx)?;
    cu::progress!(bar, "building");
    package.build(&ctx)?;
    cu::progress!(bar, "installing");
    package.install(&ctx)?;
    cu::progress!(bar, "configuring");
    ctx.items_mut()?.remove_package(pkg.to_str())?;
    package.configure(&ctx)?;
    ctx.items_mut()?.rebuild_items(Some(&bar))?;
    cu::progress!(bar, "cleaning");
    package.clean(&ctx)?;

    cu::progress!(bar, "verifying");
    match package.verify(&ctx)? {
        Verified::UpToDate => {
            bar.done();
            if let Some(mut x) = backup_guard.take() {
                x.clear();
            }
        }
        _ => {
            cu::bail!("verification failed after installation");
        }
    }
    drop(backup_guard);

    Ok(ctx)
}
