use enum_map::EnumMap;
use enumset::EnumSet;
use registry::{BinId, Context, PkgId};
use cu::pre::*;

mod install_cache;
use install_cache::InstallCache;

pub fn parse_pkgs(idents: &[String]) -> cu::Result<EnumSet<PkgId>> {
    let mut pkgs = EnumSet::new();
    for ident in idents  {
        let pkg = cu::check!(PkgId::from_str(ident), "cannot find package '{ident}'")?;
        pkgs.insert(pkg);
    }
    Ok(pkgs)
}


pub fn build_sync_graph(
    pkgs: EnumSet<PkgId>, 
    installed: &InstallCache
) -> cu::Result<Vec<PkgId>> {
    let mut sync_pkgs = EnumSet::new();
    let mut provider_selection = EnumMap::default();
    for pkg_id in pkgs {
        cu::check!(collect_dependencies(pkg_id, installed, &mut sync_pkgs, &mut provider_selection), "failed to collect dependencies")?;
    }
    loop {
        let len_before = sync_pkgs.len();
        for pkg_id in installed.pkgs {
            let ctx = Context {pkg:pkg_id};
            let cfg_deps = pkg_id.package().config_dependencies(&ctx);
            // if 
            for cfg_id in cfg_deps {
                if sync_pkgs.contains(cfg_id) {
                    sync_pkgs.insert(pkg_id);
                    break;
                }
            }
        }
        if sync_pkgs.len() == len_before {
            break;
        }
    }
    // always sync core-pseudo
    sync_pkgs.insert(PkgId::CorePseudo);

    // check if newly installed will cause conflict
    let new_pkgs = sync_pkgs.difference(installed.pkgs);
    cu::check!(installed.check_conflicts(new_pkgs), "there are conflicts in new package(s) to install")?;

    let graph = resolve_sync_order(sync_pkgs, &provider_selection)?;
    Ok(graph)
}

#[cu::error_ctx("failed to determine sync order")]
pub fn resolve_sync_order(
    pkgs: EnumSet<PkgId>, 
    bin_providers: &EnumMap<BinId, Option<PkgId>>,
) -> cu::Result<Vec<PkgId>> {
    let mut remaining = pkgs;
    let mut out = Vec::with_capacity(pkgs.len());
    while !remaining.is_empty() {
        let mut next_to_add = EnumSet::new();
        'outer: for pkg_id in remaining {
            // check bin deps
            let ctx = Context {pkg: pkg_id};
            let bin_deps = pkg_id.package().binary_dependencies(&ctx);
            for bin_id in bin_deps {
                let Some(provider) = bin_providers[bin_id] else {
                    cu::bail!("did not resolve provider for binary '{bin_id}'");
                };
                if provider == pkg_id {
                    cu::bail!("package '{pkg_id}' depends on the binary '{bin_id}' which itself provides.");
                }
                if remaining.contains(provider) {
                    // not all bin deps added
                    continue 'outer;
                }
            }
            // check config deps
            let cfg_deps = pkg_id.package().config_dependencies(&ctx);
            for cfg_id in cfg_deps {
                if remaining.contains(cfg_id) {
                    // not all cfg deps added
                    continue 'outer;
                }
            }
            next_to_add.insert(pkg_id);
        }
        if next_to_add.is_empty() {
            let pkgs_string = remaining.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ");
            cu::bail!("the order of the remaining packages cannot be determined: [ {pkgs_string} ]");
        }
        out.extend(next_to_add);
        remaining.remove_all(next_to_add);
    }
    Ok(out)
}

#[cu::error_ctx("when collecting dependencies for package '{pkg}'")]
pub fn collect_dependencies(
    pkg: PkgId, 
    installed: &InstallCache,
    out_pkgs: &mut EnumSet<PkgId>,
    provider_selection: &mut EnumMap<BinId, Option<PkgId>>
) -> cu::Result<()> {
    if !out_pkgs.insert(pkg) {
        // pkg is already added, meaning its dependencies
        // are all processed
        return Ok(());
    }
    let context = Context {pkg};
    let bin_deps = pkg.package().binary_dependencies(&context);
    for bin_id in bin_deps {
        let provider = select_provider(provider_selection, bin_id, installed)?;
        collect_dependencies(
            provider, installed, out_pkgs, provider_selection
        )?;
    }
    let cfg_deps = pkg.package().config_dependencies(&context);
    for cfg_id in cfg_deps {
        if installed.pkgs.contains(cfg_id) {
            collect_dependencies(
                cfg_id, installed, out_pkgs, provider_selection
            )?;
        }
    }

    Ok(())
}

#[cu::error_ctx("failed to select provider for binary '{bin_id}'")]
pub fn select_provider(
    provider_selection: &mut EnumMap<BinId, Option<PkgId>>,
    bin_id: BinId,
    installed: &InstallCache
) -> cu::Result<PkgId> {
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
    cu::ensure!(!providers.is_empty(), "no provider found for binary '{bin_id}'");

    // if there is only one provider for the binary, use that pkg
    if providers.len() == 1 {
        let pkg_id = providers.into_iter().next().unwrap();
        provider_selection[bin_id] = Some(pkg_id);
        cu::debug!("found provider for '{bin_id}': '{pkg_id}'");
        return Ok(pkg_id);
    }

    // prompt for a provider
    let mut error: Option<String> = None;

    let pkg_id = loop {
        cu::hint!("please select a provider for binary '{bin_id}':");
        for (i, provider) in providers.iter().enumerate() {
            cu::print!("{}  {}: {}", i+1, provider.to_str(), provider.package().short_desc);
        }
        if let Some(e) = error {
            cu::error!("{e}");
        }
        let answer = cu::prompt!("enter a number: ")?;
        let Ok(answer) = cu::parse::<usize>(&answer) else {
            error = Some("please enter a number for the provider".to_string());
            continue;
        };
        if answer == 0 {
            error = Some(format!("number too small: {answer}"));
            continue;
        }
        let pkg_id = providers.iter().skip(answer-1).next();
        let Some(pkg_id) = pkg_id else {
            error = Some(format!("number too large: {answer}"));
            continue;
        };
        break pkg_id;
    };
    provider_selection[bin_id] = Some(pkg_id);
    cu::debug!("user selected provider for '{bin_id}': '{pkg_id}'");
    Ok(pkg_id)
}
