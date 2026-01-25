use cu::pre::*;
use enum_map::EnumMap;
use enumset::EnumSet;
use registry::{BinId, PkgId};

mod install_cache;
pub use install_cache::InstallCache;

pub fn parse_pkgs(idents: &[String]) -> cu::Result<EnumSet<PkgId>> {
    let mut pkgs = EnumSet::new();
    for ident in idents {
        let pkg = cu::check!(PkgId::from_str(ident), "cannot find package '{ident}'")?;
        pkgs.insert(pkg);
    }
    Ok(pkgs)
}

pub fn build_remove_graph(
    pkgs: EnumSet<PkgId>,
    installed: &InstallCache,
    provider_selection: &mut EnumMap<BinId, Option<PkgId>>,
) -> cu::Result<Vec<PkgId>> {
    cu::debug!("building remove graph for {pkgs}");
    let mut remaining = pkgs;
    let mut updated_installed = installed.clone();
    let mut out = Vec::with_capacity(pkgs.len());
    while !remaining.is_empty() {
        let mut next_to_remove = EnumSet::new();
        for pkg_id in remaining {
            // make a copy of the current state of the install cache
            // so we can temporarily remove the package
            let mut temp_installed = updated_installed.clone();
            // assume the package is removed
            temp_installed.remove(pkg_id);
            // build a sync graph for the remaining packages
            let sync_graph = cu::check!(
                build_sync_graph(temp_installed.pkgs, &temp_installed, provider_selection),
                "failed to resolve sync graph when removing '{pkg_id}'"
            )?;
            // if the new sync graph contains the package to remove,
            // it's not ready to be removed yet
            if sync_graph.contains(&pkg_id) {
                continue;
            }
            next_to_remove.insert(pkg_id);
            updated_installed.remove(pkg_id);
            out.push(pkg_id);
        }
        if next_to_remove.is_empty() {
            let pkgs_string = remaining
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            cu::bail!(
                "cannot remove the following packages because other packages depend on them: [ {pkgs_string} ]"
            );
        }
        remaining.remove_all(next_to_remove);
    }
    Ok(out)
}

pub fn build_sync_graph(
    pkgs: EnumSet<PkgId>,
    installed: &InstallCache,
    provider_selection: &mut EnumMap<BinId, Option<PkgId>>,
) -> cu::Result<Vec<PkgId>> {
    cu::debug!("building sync graph for {pkgs}");
    let mut sync_pkgs = EnumSet::new();
    for pkg_id in pkgs {
        cu::check!(
            collect_dependencies(pkg_id, installed, &mut sync_pkgs, provider_selection),
            "failed to collect dependencies"
        )?;
    }
    let sync_pkgs = resolve_config_pkgs(sync_pkgs, EnumSet::new(), &installed);

    // check if newly installed will cause conflict
    let new_pkgs = sync_pkgs.difference(installed.pkgs);
    cu::check!(
        installed.check_conflicts(new_pkgs),
        "there are conflicts in new package(s) to install"
    )?;

    let graph = resolve_sync_order(sync_pkgs, &provider_selection)?;
    Ok(graph)
}

/// Resolve packages that should be added to sync_pkgs for config
///
/// `sync_pkgs` are packages that will be synced. `seed_pkgs` are packages that changed,
/// but will not be synced.
///
/// Return a superset of `sync_pkgs`. Only packages in `installed.pkgs` may be added
pub fn resolve_config_pkgs(
    mut sync_pkgs: EnumSet<PkgId>,
    seed_pkgs: EnumSet<PkgId>,
    installed: &InstallCache,
) -> EnumSet<PkgId> {
    loop {
        let len_before = sync_pkgs.len();
        for pkg_id in installed.pkgs {
            let cfg_deps = pkg_id.package().config_dependencies();
            for cfg_id in cfg_deps {
                if seed_pkgs.contains(cfg_id) || sync_pkgs.contains(cfg_id) {
                    sync_pkgs.insert(pkg_id);
                    break;
                }
            }
        }
        if sync_pkgs.len() == len_before {
            break;
        }
    }
    sync_pkgs
}

#[cu::context("failed to determine sync order")]
pub fn resolve_sync_order(
    pkgs: EnumSet<PkgId>,
    bin_providers: &EnumMap<BinId, Option<PkgId>>,
) -> cu::Result<Vec<PkgId>> {
    let mut remaining = pkgs;
    let mut out = Vec::with_capacity(pkgs.len() + 1);
    // always sync core-pseudo first
    remaining.remove(PkgId::Core);
    out.push(PkgId::Core);
    while !remaining.is_empty() {
        let mut next_to_add = EnumSet::new();
        'outer: for pkg_id in remaining {
            // check bin deps
            let bin_deps = pkg_id.package().binary_dependencies();
            for bin_id in bin_deps {
                let Some(provider) = bin_providers[bin_id] else {
                    cu::bail!("did not resolve provider for binary '{bin_id}'");
                };
                if provider == pkg_id {
                    cu::bail!(
                        "package '{pkg_id}' depends on the binary '{bin_id}' which itself provides."
                    );
                }
                if remaining.contains(provider) {
                    // not all bin deps added
                    continue 'outer;
                }
            }
            // check config deps
            let cfg_deps = pkg_id.package().config_dependencies();
            for cfg_id in cfg_deps {
                if remaining.contains(cfg_id) {
                    // not all cfg deps added
                    continue 'outer;
                }
            }
            next_to_add.insert(pkg_id);
        }
        if next_to_add.is_empty() {
            let pkgs_string = remaining
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            cu::bail!(
                "the order of the remaining packages cannot be determined: [ {pkgs_string} ]"
            );
        }
        out.extend(next_to_add);
        remaining.remove_all(next_to_add);
    }
    Ok(out)
}

#[cu::context("when collecting dependencies for package '{pkg}'")]
pub fn collect_dependencies(
    pkg: PkgId,
    installed: &InstallCache,
    out_pkgs: &mut EnumSet<PkgId>,
    provider_selection: &mut EnumMap<BinId, Option<PkgId>>,
) -> cu::Result<()> {
    if !out_pkgs.insert(pkg) {
        // pkg is already added, meaning its dependencies
        // are all processed
        return Ok(());
    }
    let bin_deps = pkg.package().binary_dependencies();
    cu::debug!("bin_deps for '{pkg}': {bin_deps}");
    for bin_id in bin_deps {
        let provider = select_provider(provider_selection, bin_id, installed)?;
        collect_dependencies(provider, installed, out_pkgs, provider_selection)?;
    }
    let cfg_deps = pkg.package().config_dependencies();
    for cfg_id in cfg_deps {
        if installed.pkgs.contains(cfg_id) {
            collect_dependencies(cfg_id, installed, out_pkgs, provider_selection)?;
        }
    }

    Ok(())
}

#[cu::context("failed to select provider for binary '{bin_id}'")]
pub fn select_provider(
    provider_selection: &mut EnumMap<BinId, Option<PkgId>>,
    bin_id: BinId,
    installed: &InstallCache,
) -> cu::Result<PkgId> {
    use std::fmt::Write as _;

    if let Some(pkg_id) = provider_selection[bin_id] {
        return Ok(pkg_id);
    }
    // if the binary is provided by an installed package, use that
    if let Some(pkg_id) = installed.bins[bin_id] {
        cu::debug!("found installed provider for '{bin_id}': '{pkg_id}'");
        provider_selection[bin_id] = Some(pkg_id);
        return Ok(pkg_id);
    }

    let providers = bin_id.providers();
    if providers.is_empty() {
        cu::bail!("no provider found for binary '{bin_id}'");
    }

    // if there is only one provider for the binary, use that pkg
    if providers.len() == 1 {
        let pkg_id = providers.into_iter().next().unwrap();
        provider_selection[bin_id] = Some(pkg_id);
        cu::debug!("found provider for '{bin_id}': '{pkg_id}'");
        return Ok(pkg_id);
    }

    // prompt for a provider
    let mut prompt = String::new();
    let _ = writeln!(prompt, "please select a provider for binary '{bin_id}':");
    let pkg_width = providers
        .iter()
        .map(|p| p.to_str().len())
        .max()
        .unwrap_or(1);
    for (i, provider) in providers.iter().enumerate() {
        let _ = writeln!(
            prompt,
            "{}  {:>width$}: {}",
            i + 1,
            provider.to_str(),
            provider.package().short_desc,
            width = pkg_width
        );
    }
    let _ = write!(prompt, "--- enter a number:");
    let mut pkg_id = PkgId::Core;
    cu::prompt(prompt)
        .validate_with(|answer| {
            let Ok(answer) = cu::parse::<usize>(&answer) else {
                cu::error!("please enter a number for the provider");
                return Ok(false);
            };
            if answer == 0 {
                cu::error!("number to small!");
                return Ok(false);
            }
            let pkg = providers.iter().skip(answer - 1).next();
            let Some(pkg) = pkg else {
                cu::error!("number too large: {answer} (max {})", providers.len());
                return Ok(false);
            };
            pkg_id = pkg;
            Ok(true)
        })
        .or_cancel()
        .run()?;

    provider_selection[bin_id] = Some(pkg_id);
    cu::debug!("user selected provider for '{bin_id}': '{pkg_id}'");
    Ok(pkg_id)
}
