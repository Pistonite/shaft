use std::cell::{RefCell, RefMut};
use std::path::PathBuf;
use std::sync::Arc;

use corelib::hmgr::{self, Item, ItemMgr};
use cu::pre::*;

use crate::PkgId;

/// Context passed to package functions
pub struct Context {
    /// The id of the package being operated on
    pub pkg: PkgId,
    /// Shim config
    items: RefCell<ItemMgr>,
    bar: Option<Arc<cu::ProgressBar>>,
}
impl Context {
    pub fn new(items: ItemMgr) -> Self {
        Self {
            pkg: PkgId::CorePseudo,
            items: RefCell::new(items),
            bar: None,
        }
    }
    pub fn pkg_name(&self) -> &'static str {
        self.pkg.to_str()
    }
    pub fn items_mut(&self) -> cu::Result<RefMut<'_, ItemMgr>> {
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
}
