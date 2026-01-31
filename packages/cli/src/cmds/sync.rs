use corelib::{hmgr, ItemMgr};
use cu::pre::*;
use enumset::EnumSet;
use registry::{Context, PkgId, Stage, Verified};

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
    let items = match ItemMgr::load() {
        Ok(x) => x,
        Err(e) => {
            cu::error!("{e:?}");
            cu::warn!("marking all installed packages as dirty, as items failed to load");
            cu::warn!("this is a bug if this persists.");
            for pkg in installed.pkgs {
                installed.set_dirty(pkg, true);
            }
            let config_path = hmgr::paths::items_config_json();
            cu::fs::remove(config_path)?;
            ItemMgr::load()?
        }
    };
    let graph = graph::build_sync_graph(pkgs, &installed, &mut Default::default())?;
    match graph.len() {
        1 => cu::info!("syncing 1 package..."),
        x => cu::info!("syncing {x} packages..."),
    }
    let mut ctx = Context::new(items);
    for pkg in installed.pkgs {
        ctx.set_installed(pkg, true);
    }

    for (i, pkg) in graph.iter().copied().enumerate() {
        ctx.pkg = pkg;
        let result = do_sync_package(ctx, installed);
        let result = cu::check!(result, "failed to sync '{pkg}'")?;
        ctx = result.1;
        ctx.set_bar(None);
        installed.add(pkg)?;
        ctx.set_installed(pkg, true);
        // dirty the config of inverted config dependencies
        if !matches!(result.0, SyncType::UpToDate) {
            for pkg2 in graph.iter().skip(i + 1).copied() {
                if pkg2.package().config_dependencies().contains(pkg) {
                    installed.set_dirty(pkg2, true);
                }
            }
        }
        installed.save()?;
    }

    Ok(())
}

fn do_sync_package(
    mut ctx: Context,
    installed: &mut InstallCache,
) -> cu::Result<(SyncType, Context)> {
    let pkg = ctx.pkg;
    let package = ctx.pkg.package();
    ctx.stage.set(Stage::Verify);

    let sync_type = match package.verify(&ctx)? {
        Verified::NotInstalled => SyncType::Full,
        Verified::NotUpToDate => SyncType::FullWithBackup,
        Verified::UpToDate => {
            // if the pkg is not installed yet (meaning never configured),
            // configure it even if the binaries are installed already
            //
            // and reconfigure if the config is dirtied
            if !installed.pkgs.contains(pkg) || installed.is_dirty(pkg) {
                SyncType::Config
            } else {
                SyncType::UpToDate
            }
        }
    };

    let (bar, mut backup_guard) = match sync_type {
        SyncType::UpToDate => {
            cu::info!("up to date: '{pkg}'");
            return Ok((sync_type, ctx));
        }
        SyncType::Config => {
            cu::debug!("sync type for '{pkg}': config");
            let bar = cu::progress(format!("config '{pkg}'")).spawn();
            ctx.set_bar(Some(&bar));
            (bar, None)
        }
        SyncType::FullWithBackup => {
            cu::debug!("sync type for '{pkg}': full-backup");
            let bar = cu::progress(format!("sync '{pkg}'")).spawn();
            ctx.set_bar(Some(&bar));

            cu::progress!(bar, "backup");
            ctx.stage.set(Stage::Backup);
            (bar, Some(package.backup_guard(&ctx)?))
        }
        SyncType::Full => {
            cu::debug!("sync type for '{pkg}': full");
            let bar = cu::progress(format!("sync '{pkg}'")).spawn();
            ctx.set_bar(Some(&bar));
            (bar, None)
        }
    };

    if !matches!(sync_type, SyncType::Config) {
        cu::progress!(bar, "downloading");
        ctx.stage.set(Stage::Download);
        package.download(&ctx)?;

        cu::progress!(bar, "building");
        ctx.stage.set(Stage::Build);
        package.build(&ctx)?;

        cu::progress!(bar, "installing");
        ctx.stage.set(Stage::Install);
        package.install(&ctx)?;
    }

    cu::progress!(bar, "configuring");
    ctx.stage.set(Stage::Configure);
    ctx.items_mut()?.remove_package(pkg.to_str(), Some(&bar))?;
    package.configure(&ctx)?;
    ctx.items_mut()?.rebuild_items(Some(&bar))?;
    installed.set_dirty(pkg, false);

    cu::progress!(bar, "cleaning");
    ctx.stage.set(Stage::Clean);
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
            cu::hint!(
                r"verification failed after installation
this could happen for one of the following reasons:
- an external package installer modified the environment variable
- an external package installer has an outdated registry/mirrors
  - for example, run `update-mirrors` on arch linux
- the shaft registry has an incorrect version of the package

please try restarting all terminal processes and shell sessions,
and update the registries/mirrors of external package managers
such as pacman and cargo.
"
            );
            cu::bail!("failed to verify '{pkg}' after installation");
        }
    }
    drop(backup_guard);

    Ok((sync_type, ctx))
}

enum SyncType {
    /// already up-to-date, nothing to do
    UpToDate,
    /// Just run the config stage to refresh the config
    Config,
    /// Full sync - download and install
    Full,
    /// Full sync - download and install, and also backup the old installation
    FullWithBackup,
}
