use std::collections::BTreeMap;

use enum_map::EnumMap;
use enumset::EnumSet;
use registry::{BinId, PkgId};

use cu::pre::*;

#[derive(Default)]
pub struct InstallCache {
    /// Set of packages installed
    pub pkgs: EnumSet<PkgId>,
    /// Binaries available mapping to the package that provides it
    pub bins: EnumMap<BinId, Option<PkgId>>,
}

impl InstallCache {
    #[cu::error_ctx("failed to load install cache")]
    pub fn load() -> cu::Result<Self> {
        cu::trace!("loading install cache");
        let path = op::home::home().join("install_cache.json");
        if !path.exists() {
            cu::debug!("no install cache");
            return Ok(Default::default());
        }
        let content = cu::fs::read_string(path)?;
        let install_cache: InstallCacheJson = json::parse(&content)?;
        cu::debug!("install cache loaded: {install_cache:?}");
        Ok(install_cache.into())
    }

    #[cu::error_ctx("failed to save install cache")]
    pub fn save(&self) -> cu::Result<()> {
        let path = op::home::home().join("install_cache.json");
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
        let mut bins: EnumMap<BinId, Option<PkgId>> = EnumMap::default();
        for (bin, pkg) in &value.bins {
            let Some(bin_id) = BinId::from_str(bin) else {
                continue;
            };
            let Some(pkg_id) = PkgId::from_str(pkg) else {
                continue;
            };
            // ensures the package still provides the binary
            if !pkg_id.package().binaries().contains(bin_id) {
                continue;
            }
            bins[bin_id] = Some(pkg_id);
        }
        Self { pkgs, bins }
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
        let bins = value
            .bins
            .iter()
            .filter_map(|(k, v)| Some((k.to_string(), v.as_ref().copied()?.to_string())))
            .collect();
        Self { pkgs, bins }
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
    /// Binaries available mapping to the package that provides it
    pub bins: BTreeMap<String, String>,
}
