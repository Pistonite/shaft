use corelib::ItemMgr;
use cu::pre::*;
use enumset::EnumSet;
use registry::{Context, PkgId, Verified};

use crate::graph::{self, InstallCache};

pub fn remove(packages: &[String]) -> cu::Result<()> {
    let pkgs = graph::parse_pkgs(packages)?;
    let mut installed = InstallCache::load()?;
    let pkgs = rectify_pkgs_to_remove(pkgs, &installed);
    cu::ensure!(
        !pkgs.is_empty(),
        "please specify packages to remove, see `shaft remove -h`"
    );

    let graph = graph::build_remove_graph(pkgs, &installed, &mut Default::default())?;
    match graph.len() {
        0 => cu::bail!("nothing to do"),
        1 => cu::info!("removing 1 package..."),
        x => cu::info!("removing {x} packages..."),
    }

    let items = ItemMgr::load()?;

    // check precondition for each package
    let mut to_uninstall = Vec::with_capacity(graph.len());
    let mut ctx = Context::new(items);
    for pkg in &graph {
        let pkg = *pkg;
        ctx.pkg = pkg;
        let package = pkg.package();
        match package.verify(&ctx)? {
            Verified::NotInstalled => {
                cu::warn!("'{pkg}' is not installed, skipping");
            }
            _ => {
                package.pre_uninstall(&ctx)?;
                to_uninstall.push(pkg);
            }
        }
    }

    let len = to_uninstall.len();
    let uninstalled: EnumSet<_> = to_uninstall.iter().copied().collect();
    for pkg in to_uninstall {
        ctx.pkg = pkg;
        ctx = cu::check!(do_remove_package(ctx), "failed to remove '{pkg}'")?;
        ctx.set_bar(None);
        installed.remove(pkg);
        installed.save()?;
    }

    // rebuild items if needed (if any package removed their items)
    {
        let bar = cu::progress("rebuilding items").spawn();
        ctx.items_mut()?.rebuild_items(Some(&bar))?;
        bar.done();
    }

    cu::info!("removed {len} packages, configuring...");
    let sync_pkgs = graph::resolve_config_pkgs(EnumSet::new(), uninstalled, &installed);
    cu::check!(
        super::sync_pkgs(sync_pkgs, &mut installed),
        "failed to configure packages after removing"
    )?;

    Ok(())
}

fn rectify_pkgs_to_remove(pkgs: EnumSet<PkgId>, installed: &InstallCache) -> EnumSet<PkgId> {
    let mut out = EnumSet::new();
    // check if each package is installed
    for pkg in pkgs {
        if !installed.pkgs.contains(pkg) {
            cu::warn!("'{pkg}' is not in install cache, sync it first if it's installed.");
            continue;
        }
        out.insert(pkg);
    }
    out
}

fn do_remove_package(mut ctx: Context) -> cu::Result<Context> {
    let pkg = ctx.pkg;
    let package = pkg.package();
    let bar = cu::progress(format!("remove '{pkg}'")).spawn();
    ctx.set_bar(Some(&bar));

    cu::progress!(bar, "backup");
    let mut backup_guard = package.backup_guard(&ctx)?;
    cu::progress!(bar, "cleaning");
    package.clean(&ctx)?;
    cu::progress!(bar, "uninstalling");
    package.uninstall(&ctx)?;

    cu::progress!(bar, "verifying");
    match package.verify(&ctx)? {
        Verified::NotInstalled => {
            backup_guard.clear();
            drop(backup_guard);
            bar.done();
            return Ok(ctx);
        }
        _ => {
            cu::error!("uninstalling not successful for '{pkg}', restoring...");
        }
    }
    drop(backup_guard);
    drop(bar);
    cu::warn!(
        "package '{pkg}' is not removed - recommend to sync all packages to ensure a consistent state"
    );
    cu::bail!("verification failed after uninstalling '{pkg}'");
}
