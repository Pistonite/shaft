use std::cell::{RefCell, RefMut};
use std::path::PathBuf;
use std::sync::Arc;

use corelib::hmgr::{self, Item, ItemMgr};
use cu::pre::*;
use enumset::EnumSet;

use crate::PkgId;

/// Context passed to package functions
pub struct Context {
    /// The id of the package being operated on
    pub pkg: PkgId,
    pub stage: cu::Atomic<u8, Stage>,
    /// Shim config
    items: RefCell<ItemMgr>,
    bar: Option<Arc<cu::ProgressBar>>,
    installed: EnumSet<PkgId>,
}
impl Context {
    pub fn new(items: ItemMgr) -> Self {
        Self {
            pkg: PkgId::Core,
            stage: cu::Atomic::new_u8(Stage::Verify.into()),
            items: RefCell::new(items),
            bar: None,
            installed: EnumSet::default(),
        }
    }
    pub fn pkg_name(&self) -> &'static str {
        self.pkg.to_str()
    }
    pub fn items_mut(&self) -> cu::Result<RefMut<'_, ItemMgr>> {
        if self.stage.get() != Stage::Configure {
            cu::bail!("config items may only be modified during the configure stage");
        }
        cu::check!(
            self.items.try_borrow_mut(),
            "unexpected: failed to borrow items_mut"
        )
    }
    pub fn add_item(&self, item: Item) -> cu::Result<()> {
        self.items_mut()?.add_item(self.pkg.to_str(), item);
        Ok(())
    }

    pub fn set_bar(&mut self, bar: Option<&Arc<cu::ProgressBar>>) {
        self.bar = bar.cloned();
    }
    pub fn bar(&self) -> Option<Arc<cu::ProgressBar>> {
        self.bar.clone()
    }

    pub fn temp_dir(&self) -> PathBuf {
        hmgr::paths::temp_dir(self.pkg_name())
    }
    pub fn load_config_file_or_default(&self, default_config: &str) -> cu::Result<toml::Table> {
        let config_file = self.config_file();
        let Ok(config) = cu::fs::read_string(&config_file) else {
            let _ = cu::fs::write(&config_file, &default_config)?;
            let default = toml::parse(default_config)?;
            return Ok(default);
        };
        let config = cu::check!(
            toml::parse(&config),
            "failed to parse config file for '{}'",
            self.pkg
        )?;
        Ok(config)
    }
    pub fn config_file(&self) -> PathBuf {
        hmgr::paths::config_file(self.pkg_name())
    }
    pub fn install_dir(&self) -> PathBuf {
        hmgr::paths::install_dir(self.pkg_name())
    }
    pub fn install_old_dir(&self) -> PathBuf {
        hmgr::paths::install_old_dir(self.pkg_name())
    }
    /// Move HOME/install/<package> directory to HOME/install-old/<package>,
    /// if it exists. The old old will be deleted
    pub fn move_install_to_old_if_exists(&self) -> cu::Result<()> {
        let cur_install_dir = self.install_dir();
        if !cur_install_dir.exists() {
            return Ok(());
        }
        cu::debug!("moving install dir to old: '{}'", cur_install_dir.display());
        let old_install_dir = self.install_old_dir();
        let old_install_root = hmgr::paths::install_old_root();
        cu::check!(
            cu::fs::make_dir(old_install_root),
            "failed to create old install root"
        )?;
        cu::check!(
            cu::fs::rec_remove(&old_install_dir),
            "failed to remove old install dir"
        )?;
        cu::check!(
            cu::fs::rename(cur_install_dir, old_install_dir),
            "failed to move install dir to install-old"
        )?;
        Ok(())
    }
    pub fn set_installed(&mut self, pkg: PkgId, installed: bool) {
        if installed {
            self.installed.insert(pkg);
        } else {
            self.installed.remove(pkg);
        }
    }
    pub fn is_installed(&self, pkg: PkgId) -> bool {
        self.installed.contains(pkg)
    }
}

/// Stages when working with the package
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Stage {
    Verify = 0,
    Backup = 1,
    Download = 2,
    Build = 3,
    Install = 4,
    Configure = 5,
    Clean = 6,
    Uninstall = 7,
}
impl From<Stage> for u8 {
    fn from(stage: Stage) -> Self {
        stage as u8
    }
}
impl From<u8> for Stage {
    fn from(value: u8) -> Self {
        match value {
            0 => Stage::Verify,
            1 => Stage::Backup,
            2 => Stage::Download,
            3 => Stage::Build,
            4 => Stage::Install,
            5 => Stage::Configure,
            6 => Stage::Clean,
            7 => Stage::Uninstall,
            _ => panic!("invalid Stage value: {value}"),
        }
    }
}
