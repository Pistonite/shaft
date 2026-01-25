use corelib::ItemMgr;
use cu::pre::*;
use enumset::EnumSet;
use registry::{Context, PkgId, Stage, Verified};

use crate::graph::{self, InstallCache};

pub fn remove(packages: &[String], force: bool) -> cu::Result<()> {
    let pkgs = graph::parse_pkgs(packages)?;
    let mut installed = InstallCache::load()?;
    let pkgs = rectify_pkgs_to_remove(pkgs, &installed, force);
    if pkgs.is_empty() {
        cu::bail!("please specify packages to remove, see `shaft remove -h`");
    }

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
    for pkg in installed.pkgs {
        ctx.set_installed(pkg, true);
    }

    for pkg in &graph {
        let pkg = *pkg;
        let package = pkg.package();
        ctx.pkg = pkg;
        ctx.stage.set(Stage::Verify);
        match package.verify(&ctx) {
            Ok(Verified::NotInstalled) => {
                if ! force {
                    cu::warn!("'{pkg}' is not installed, skipping");
                    continue;
                }
            }
            Ok(_) => {}
            Err(e) => {
                if !force {
                    cu::rethrow!(e, "failed to verify package status (--force to bypass)");
                }
                cu::warn!("will force uninstall '{pkg}' because of error: {e:?}");
            }
        }
        package.pre_uninstall(&ctx)?;
        to_uninstall.push(pkg);
    }

    let len = to_uninstall.len();
    let uninstalled: EnumSet<_> = to_uninstall.iter().copied().collect();
    let sync_pkgs = graph::resolve_config_pkgs(EnumSet::new(), uninstalled, &installed);
    for pkg in sync_pkgs {
        installed.set_dirty(pkg, true);
    }
    installed.save()?;

    for pkg in to_uninstall {
        ctx.pkg = pkg;
        ctx = cu::check!(do_remove_package(ctx), "failed to remove '{pkg}'")?;
        ctx.set_bar(None);
        installed.remove(pkg);
        ctx.set_installed(pkg, false);
        installed.save()?;
    }

    // rebuild items if needed (if any package removed their items)
    {
        let bar = cu::progress("rebuilding items").spawn();
        ctx.stage.set(Stage::Configure);
        ctx.items_mut()?.rebuild_items(Some(&bar))?;
        bar.done();
    }

    cu::info!("removed {len} packages, configuring...");
    cu::check!(
        super::sync_pkgs(sync_pkgs, &mut installed),
        "failed to configure packages after removing"
    )?;

    Ok(())
}

fn rectify_pkgs_to_remove(pkgs: EnumSet<PkgId>, installed: &InstallCache, force: bool) -> EnumSet<PkgId> {
    let mut out = EnumSet::new();
    // check if each package is installed
    for pkg in pkgs {
        if !installed.pkgs.contains(pkg) {
            if !force {
                cu::warn!("'{pkg}' is not in install cache, sync it first if it's installed.");
                continue;
            }
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
    ctx.stage.set(Stage::Backup);
    let mut backup_guard = package.backup_guard(&ctx)?;

    cu::progress!(bar, "uninstalling");
    ctx.stage.set(Stage::Uninstall);
    package.uninstall(&ctx)?;
    ctx.stage.set(Stage::Configure);
    ctx.items_mut()?.remove_package(pkg.to_str())?;

    cu::progress!(bar, "cleaning");
    ctx.stage.set(Stage::Clean);
    package.clean(&ctx)?;

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
