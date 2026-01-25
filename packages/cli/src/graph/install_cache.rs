use std::collections::BTreeMap;

use corelib::hmgr;
use cu::pre::*;
use enum_map::EnumMap;
use enumset::EnumSet;
use registry::{BinId, PkgId};

#[derive(Debug, Default, Clone)]
pub struct InstallCache {
    /// Set of packages installed
    pub pkgs: EnumSet<PkgId>,
    /// Set of packages with dirtied configs
    pub dirty: EnumSet<PkgId>,
    /// Binaries available mapping to the package that provides it
    pub bins: EnumMap<BinId, Option<PkgId>>,
}

impl InstallCache {
    #[cu::context("failed to load install cache")]
    pub fn load() -> cu::Result<Self> {
        cu::trace!("loading install cache");
        let path = hmgr::paths::install_cache_json();
        if !path.exists() {
            cu::debug!("no install cache");
            return Ok(Default::default());
        }
        let content = cu::fs::read_string(path)?;
        let install_cache: InstallCacheJson = json::parse(&content)?;
        cu::debug!("install cache loaded: {install_cache:?}");
        Ok(install_cache.into())
    }

    #[cu::context("failed to save install cache")]
    pub fn save(&self) -> cu::Result<()> {
        let path = hmgr::paths::install_cache_json();
        let install_cache = InstallCacheJson::from(self);
        cu::fs::write_json_pretty(path, &install_cache)?;
        Ok(())
    }

    /// Check if the pkg can be installed
    pub fn check_conflicts(&self, new_pkg_ids: EnumSet<PkgId>) -> cu::Result<()> {
        // check for conflict with existing packages
        for new_pkg_id in new_pkg_ids {
            if self.pkgs.contains(new_pkg_id) {
                // already installed - no conflict
                return Ok(());
            }

            for bin_id in new_pkg_id.package().binaries() {
                if let Some(existing_pkg_id) = self.bins[bin_id] {
                    cu::bail!(
                        "package '{new_pkg_id}' provides binary '{bin_id}', which is already provided by the '{existing_pkg_id}' package currently installed."
                    );
                }
            }
        }
        // check for conflicts among new packages
        let mut new_bin_ids: EnumMap<BinId, Option<PkgId>> = EnumMap::default();
        for new_pkg_id in new_pkg_ids {
            for bin_id in new_pkg_id.package().binaries() {
                if let Some(existing_pkg_id) = new_bin_ids[bin_id] {
                    cu::bail!(
                        "package '{new_pkg_id}' and package '{existing_pkg_id}' both provide binary '{bin_id}', only one of them can be installed."
                    );
                }
                new_bin_ids[bin_id] = Some(new_pkg_id);
            }
        }

        Ok(())
    }

    #[cu::context("failed to add '{pkg}' to install cache")]
    pub fn add(&mut self, pkg: PkgId) -> cu::Result<()> {
        // sanity check
        cu::check!(
            self.check_conflicts(pkg.into()),
            "there are conflict(s) with existing packages"
        )?;
        self.pkgs.insert(pkg);
        for bin in pkg.package().binaries() {
            // this is ok because we checked for conflicts
            self.bins[bin] = Some(pkg);
        }
        Ok(())
    }

    pub fn remove(&mut self, pkg: PkgId) {
        self.dirty.remove(pkg);
        if !self.pkgs.remove(pkg) {
            // was not installed, no-op
            return;
        }
        for (_, pkg_id) in &mut self.bins {
            if *pkg_id == Some(pkg) {
                *pkg_id = None;
            }
        }
    }

    pub fn is_dirty(&self, pkg: PkgId) -> bool {
        self.dirty.contains(pkg)
    }

    pub fn set_dirty(&mut self, pkg: PkgId, dirty: bool) {
        if dirty {
            self.dirty.insert(pkg);
        } else {
            self.dirty.remove(pkg);
        }
    }
}

impl From<&InstallCacheJson> for InstallCache {
    fn from(value: &InstallCacheJson) -> Self {
        let mut pkgs = EnumSet::new();
        for name in &value.pkgs {
            let Some(pkg_id) = PkgId::from_str(name) else {
                continue;
            };
            pkgs.insert(pkg_id);
        }
        let mut dirty = EnumSet::new();
        for name in &value.dirty {
            let Some(pkg_id) = PkgId::from_str(name) else {
                continue;
            };
            if pkgs.contains(pkg_id) {
                dirty.insert(pkg_id);
            }
        }
        let mut bins: EnumMap<BinId, Option<PkgId>> = EnumMap::default();
        for (bin, pkg) in &value.bins {
            let Some(bin_id) = BinId::from_str(bin) else {
                continue;
            };
            let Some(pkg_id) = PkgId::from_str(pkg) else {
                continue;
            };
            if !pkgs.contains(pkg_id) {
                continue;
            }
            // ensures the package still provides the binary
            if !pkg_id.package().binaries().contains(bin_id) {
                continue;
            }
            bins[bin_id] = Some(pkg_id);
        }
        Self { pkgs, dirty, bins }
    }
}
impl From<InstallCacheJson> for InstallCache {
    #[inline(always)]
    fn from(value: InstallCacheJson) -> Self {
        (&value).into()
    }
}

impl From<&InstallCache> for InstallCacheJson {
    fn from(value: &InstallCache) -> Self {
        let pkgs = value.pkgs.iter().map(|x| x.to_string()).collect();
        let dirty = value.dirty.iter().map(|x| x.to_string()).collect();
        let bins = value
            .bins
            .iter()
            .filter_map(|(k, v)| Some((k.to_string(), v.as_ref().copied()?.to_string())))
            .collect();
        Self { pkgs, dirty, bins }
    }
}
impl From<InstallCache> for InstallCacheJson {
    #[inline(always)]
    fn from(value: InstallCache) -> Self {
        (&value).into()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct InstallCacheJson {
    /// List of packages installed
    pub pkgs: Vec<String>,
    /// List of packages with dirtied (edited) config
    #[serde(default)]
    pub dirty: Vec<String>,
    /// Binaries available mapping to the package that provides it
    pub bins: BTreeMap<String, String>,
}
