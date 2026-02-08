use corelib::ItemMgr;
use enumset::EnumSet;
use itertools::Itertools as _;
use registry::{BinId, Context, PkgId};

use crate::graph::InstallCache;

/// Query for package information
pub fn info(
    query: &str,
    search: bool,
    installed_only: bool,
    binary_only: bool,
    package_only: bool,
    machine_mode: bool,
) -> cu::Result<bool> {
    let query = query.trim();
    cu::debug!("querying for '{query}'");
    if !installed_only && query.is_empty() {
        cu::bail!("please provide a query string");
    }

    if query.is_empty() {
        return show_results(
            None,
            EnumSet::all(),
            installed_only,
            false,
            None,
            machine_mode,
        );
    }

    if !search {
        return info_exact(
            query,
            installed_only,
            binary_only,
            package_only,
            machine_mode,
        );
    }

    let query_mode = QueryMode::parse(query)?;

    if package_only {
        let exact_pkg = PkgId::from_str(query);
        let mut pkgs = EnumSet::default();
        for pkg in EnumSet::<PkgId>::all() {
            if !query_mode.matches(pkg.to_str()) {
                continue;
            }
            pkgs.insert(pkg);
        }
        return show_results(exact_pkg, pkgs, installed_only, false, None, machine_mode);
    }

    if binary_only {
        let pkg = PkgId::from_str(query);
        let exact_bin = BinId::from_str(query);
        let mut pkgs = exact_bin.map(|x| x.providers()).unwrap_or_default();
        let exact_pkg = pkg.filter(|x| pkgs.contains(*x));
        for bin in EnumSet::<BinId>::all() {
            if !query_mode.matches(bin.to_str()) {
                continue;
            }
            pkgs.extend(bin.providers());
        }
        return show_results(
            exact_pkg,
            pkgs,
            installed_only,
            true,
            exact_bin,
            machine_mode,
        );
    }

    let exact_pkg = PkgId::from_str(query);
    let exact_bin = BinId::from_str(query);
    let mut pkgs = EnumSet::default();
    for pkg in EnumSet::<PkgId>::all() {
        if !query_mode.matches(pkg.to_str()) {
            continue;
        }
        pkgs.insert(pkg);
    }
    for bin in EnumSet::<BinId>::all() {
        if !query_mode.matches(bin.to_str()) {
            continue;
        }
        pkgs.extend(bin.providers());
    }
    if let Some(pkg) = exact_pkg {
        pkgs.remove(pkg);
    }
    show_results(
        exact_pkg,
        pkgs,
        installed_only,
        false,
        exact_bin,
        machine_mode,
    )
}

fn info_exact(
    query_exact: &str,
    installed_only: bool,
    binary_only: bool,
    package_only: bool,
    machine_mode: bool,
) -> cu::Result<bool> {
    let the_pkg = PkgId::from_str(query_exact);
    let the_bin = BinId::from_str(query_exact);

    if package_only {
        let Some(the_pkg) = the_pkg else {
            cu::error!("did not find package '{query_exact}'");
            if let Some(the_bin) = the_bin {
                cu::hint!("there is a binary with the same name, try `shaft info -b {the_bin}`");
            } else {
                cu::hint!("no results found, try search mode: `shaft info -sb {query_exact}`");
            }
            return Ok(false);
        };
        let installed = InstallCache::load()?;
        display_pkg_detail(&installed, the_pkg, machine_mode);
        return Ok(true);
    }

    if binary_only {
        let Some(the_bin) = the_bin else {
            cu::error!("did not find binary '{query_exact}'");
            if let Some(the_pkg) = the_pkg {
                cu::hint!("there is a package with the same name, try `shaft info {the_pkg}`");
            }
            return Ok(false);
        };
        let pkgs = the_bin.providers();
        if pkgs.is_empty() {
            cu::warn!("did not find any package the provides '{the_bin}' on the current platform");
            if let Some(the_pkg) = the_pkg {
                cu::hint!("there is a package with the same name, try `shaft info {the_pkg}`");
            }
            return Ok(false);
        }
        let exact_pkg = match the_pkg {
            Some(the_pkg) => {
                if !pkgs.contains(the_pkg) {
                    cu::hint!(
                        "there is also a package with the same name but does not provide the binary '{the_bin}'"
                    );
                    None
                } else {
                    Some(the_pkg)
                }
            }
            _ => None,
        };
        return show_results(
            exact_pkg,
            pkgs,
            installed_only,
            true,
            Some(the_bin),
            machine_mode,
        );
    }

    let pkgs = the_bin.map(|x| x.providers()).unwrap_or_default();
    let result = show_results(the_pkg, pkgs, installed_only, false, the_bin, machine_mode)?;
    if !result {
        cu::hint!("no results found, try search mode: `shaft info -s {query_exact}`");
    }
    Ok(result)
}

fn show_results(
    mut exact_pkg: Option<PkgId>,
    mut rest_pkgs: EnumSet<PkgId>,
    installed_only: bool,
    binary_only: bool,
    binary_result: Option<BinId>,
    machine_mode: bool,
) -> cu::Result<bool> {
    let installed = InstallCache::load()?;
    if let Some(pkg) = exact_pkg {
        rest_pkgs.remove(pkg);
    }
    if installed_only {
        if let Some(pkg) = exact_pkg {
            if !installed.pkgs.contains(pkg) {
                exact_pkg = None;
            }
        }
        for pkg in rest_pkgs {
            if !installed.pkgs.contains(pkg) {
                rest_pkgs.remove(pkg);
            }
        }
    }
    match (exact_pkg, rest_pkgs.len()) {
        (None, 0) => {
            return Ok(false);
        }
        (None, 1) => {
            if let Some(bin) = binary_result {
                cu::info!("found 1 provider for the binary '{bin}'");
            }
            display_pkg_detail(&installed, rest_pkgs.iter().next().unwrap(), machine_mode);
        }
        (None, x) => {
            if let Some(bin) = binary_result {
                cu::info!("found {x} providers for the binary '{bin}'");
            } else {
                cu::info!("found {x} packages");
            }
            display_pkgs_summary(&installed, rest_pkgs, machine_mode);
        }
        (Some(pkg), 0) => {
            display_pkg_detail(&installed, pkg, machine_mode);
        }
        (Some(pkg), x) => {
            let len = x + 1;
            if let Some(bin) = binary_result {
                if binary_only {
                    cu::info!("found {len} providers for the binary '{bin}'");
                    display_pkg_detail(&installed, pkg, machine_mode);
                    cu::info!("the following packages also provide the binary:");
                    display_pkgs_summary(&installed, rest_pkgs, machine_mode);
                } else {
                    display_pkg_detail(&installed, pkg, machine_mode);
                    cu::info!("other packages that matched:");
                    display_pkgs_summary(&installed, rest_pkgs, machine_mode);
                }
            } else {
                cu::info!("found {len} packages");
                display_pkg_detail(&installed, pkg, machine_mode);
                cu::info!("other packages that matched:");
                display_pkgs_summary(&installed, rest_pkgs, machine_mode);
            }
        }
    }
    Ok(true)
}

fn display_pkg_detail(installed: &InstallCache, pkg: PkgId, machine_mode: bool) {
    if machine_mode {
        println!("{pkg}");
        return;
    }
    let package = pkg.package();
    if !package.enabled() {
        cu::warn!("package '{pkg}' is disabled on the current platform");
    }
    cu::hint!("=== [package: {pkg}] ============");
    let mut desc = package.short_desc.to_string();
    if !package.long_desc.is_empty() {
        desc.push('\n');
        desc.push_str(package.long_desc);
        desc.push('\n');
    }
    cu::hint!("{desc}");
    let bins = package.binaries();
    let bins_str = bins.iter().map(|x| x.to_str()).join(", ");
    if bins.len() == 1 {
        cu::print!("        binary: {bins_str}");
    } else {
        cu::print!("      binaries: [{bins_str}]");
    }
    let mut ctx = Context::new(ItemMgr::default());
    ctx.pkg = pkg;
    match package.config_location(&ctx) {
        Err(e) => {
            cu::error!("  configurable: [error: {e}]");
        }
        Ok(x) => {
            cu::print!("  configurable: {}", display_bool(x.is_some()));
        }
    }
    cu::print!(
        "     installed: {}",
        display_bool(installed.pkgs.contains(pkg))
    );
    cu::print!("       dirtied: {}", display_bool(installed.is_dirty(pkg)));
    let bin_deps = package
        .binary_dependencies()
        .iter()
        .map(|x| x.to_str())
        .join(", ");
    cu::print!("      bin_deps: [{bin_deps}]");
    let cfg_deps = package
        .binary_dependencies()
        .iter()
        .map(|x| x.to_str())
        .join(", ");
    cu::print!("      cfg_deps: [{cfg_deps}]");
    cu::print!("");
}
fn display_pkgs_summary(installed: &InstallCache, pkgs: EnumSet<PkgId>, machine_mode: bool) {
    if machine_mode {
        for pkg in pkgs {
            println!("{pkg}");
        }
        return;
    }
    let package_field_width = pkgs
        .iter()
        .map(|x| x.to_str().len())
        .max()
        .unwrap_or(10)
        .max(10);
    cu::hint!(
        "{:>package_field_width$} | installed | dirtied | description\n------------------------------------------------------------",
        "package"
    );
    for pkg in pkgs {
        let package = pkg.package();
        if !package.enabled() {
            cu::warn!("package '{pkg}' is disabled on the current platform");
        }
        let is_installed = display_bool(installed.pkgs.contains(pkg));
        let is_dirtied = display_bool(installed.is_dirty(pkg));
        let desc = package.short_desc;

        cu::print!("{pkg:>package_field_width$} | {is_installed:>9} | {is_dirtied:>7} | {desc}")
    }
}

fn display_bool(x: bool) -> &'static str {
    if x { "yes" } else { "no" }
}

#[derive(Clone, Copy)]
enum QueryMode<'a> {
    All,
    Substring(&'a str),
    StartsWith(&'a str),
    EndsWith(&'a str),
    StartsWithEndsWith(&'a str, &'a str),
}

impl<'a> QueryMode<'a> {
    fn parse(query: &'a str) -> cu::Result<Self> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(Self::All);
        }
        if query.starts_with('*') {
            let query = query.trim_start_matches('*').trim_start();
            if query.ends_with('*') {
                let query = query.trim_end_matches('*').trim_end();
                if query.contains('*') {
                    cu::bail!("invalid query: {query:?}");
                }
                return Ok(Self::Substring(query));
            }
            if query.contains('*') {
                cu::bail!("invalid query: {query:?}");
            }
            return Ok(Self::EndsWith(query));
        }
        if query.ends_with('*') {
            let query = query.trim_end_matches('*').trim_end();
            if query.contains('*') {
                cu::bail!("invalid query: {query:?}");
            }
            return Ok(Self::StartsWith(query));
        }
        let Some((prefix, suffix)) = query.split_once('*') else {
            return Ok(Self::Substring(query));
        };
        if suffix.contains('*') {
            cu::bail!("invalid query: {query:?}");
        }
        Ok(Self::StartsWithEndsWith(
            prefix.trim_end(),
            suffix.trim_start(),
        ))
    }
    fn matches(self, s: &str) -> bool {
        match self {
            QueryMode::All => true,
            QueryMode::Substring(q) => s.contains(q),
            QueryMode::StartsWith(q) => s.starts_with(q),
            QueryMode::EndsWith(q) => s.ends_with(q),
            QueryMode::StartsWithEndsWith(prefix, suffix) => {
                s.starts_with(prefix) && s.ends_with(suffix)
            }
        }
    }
}
