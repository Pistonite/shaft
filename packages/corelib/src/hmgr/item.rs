#[cfg(windows)]
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use cu::pre::*;
use shaftim_build::{ShimCommand, ShimConfig};

use crate::hmgr::env::Env;
use crate::{bin_name, epkg, hmgr, opfs};

#[derive(Default)]
pub struct ItemMgr {
    items: Vec<ItemEntry>,
    skip_reinvocation: bool,
    dirty: bool,
    link_dirty: bool,
    shim_dirty: bool,
    env: Env,
}

impl ItemMgr {
    #[cu::context("failed to load installed items")]
    pub fn load() -> cu::Result<Self> {
        let config_path = hmgr::paths::items_config_json();
        let Ok(items) = cu::fs::read_string(config_path) else {
            return Ok(Self {
                items: vec![],
                skip_reinvocation: false,
                dirty: true,
                shim_dirty: true,
                link_dirty: true,
                env: Env::new_dirty(),
            });
        };
        let items = match json::parse(&items) {
            Ok(x) => x,
            Err(e) => {
                cu::warn!(
                    "failed to load installed items; the format might have changed; sync will fix re-configure the items if that is the case."
                );
                return Err(e);
            }
        };
        Ok(Self {
            items,
            skip_reinvocation: false,
            dirty: false,
            shim_dirty: false,
            link_dirty: false,
            env: Default::default(),
        })
    }
    pub fn skip_reinvocation(&mut self, skip: bool) {
        self.skip_reinvocation = skip;
    }
    pub fn add_item(&mut self, package: &str, item: Item, priority: i32) {
        let entry = ItemEntry {
            package: package.to_string(),
            item,
            priority,
        };
        if self.items.contains(&entry) {
            return;
        }
        match &entry.item {
            Item::UserEnvVar(_, _) => {}
            Item::UserPath(_) => {}
            #[cfg(target_os = "linux")]
            Item::SessionEnvVar(_, _, _) => {}
            Item::LinkBin(_, _, _) => self.link_dirty = true,
            Item::ShimBin(_, _) => self.shim_dirty = true,
            Item::Pwsh(_) => {}
            Item::Bash(_) => {}
            Item::Zsh(_) => {}
            Item::Cmd(_) => {}
        }
        self.env.on_item_modified(&entry);
        self.dirty = true;
        self.items.push(entry);
    }

    pub fn remove_package(&mut self, package: &str) -> cu::Result<()> {
        self.remove_package_internal(Some(package))
    }

    pub fn remove_all(&mut self) -> cu::Result<()> {
        self.remove_package_internal(None)
    }

    fn remove_package_internal(
        &mut self,
        package: Option<&str>, // none for removing all
    ) -> cu::Result<()> {
        let mut bin_to_remove = vec![];
        #[cfg(windows)]
        let mut env_to_remove = BTreeMap::new();
        #[cfg(windows)]
        let mut path_to_remove = BTreeSet::new();

        // take out items to workaround borrow check
        let mut items = std::mem::take(&mut self.items);
        items.retain(|entry| {
            if package != Some(&entry.package) {
                return true;
            }
            self.env.on_item_modified(entry);
            match &entry.item {
                #[cfg(windows)]
                Item::UserEnvVar(k, v) => {
                    env_to_remove.insert(k.to_string(), v.to_string());
                }
                #[cfg(not(windows))]
                Item::UserEnvVar(_, _) => {}

                #[cfg(windows)]
                Item::UserPath(path) => {
                    path_to_remove.insert(path.to_string());
                }
                #[cfg(not(windows))]
                Item::UserPath(_) => {}

                #[cfg(target_os = "linux")]
                Item::SessionEnvVar(_, _, _) => {}

                Item::LinkBin(bin, _, _) => {
                    bin_to_remove.push(bin.to_string());
                    // removing a link does not make links dirty
                }
                Item::ShimBin(bin, _) => {
                    bin_to_remove.push(bin.to_string());
                    self.shim_dirty = true;
                }
                Item::Pwsh(_) => {}
                Item::Bash(_) => {}
                Item::Zsh(_) => {}
                Item::Cmd(_) => {}
            }
            self.dirty = true;
            false
        });
        self.items = items;

        if !bin_to_remove.is_empty() {
            let bin_root = hmgr::paths::bin_root();
            for bin in bin_to_remove {
                if let Err(e) = opfs::safe_remove_link(&bin_root.join(bin)) {
                    cu::warn!("failed to remove old link: {e}");
                }
            }
        }

        #[cfg(windows)]
        {
            for (key, value) in env_to_remove {
                if let Ok(current_value) = hmgr::windows::get_user(&key) {
                    if current_value != value {
                        cu::warn!(
                            "removing user env var '{key}', but the current value is not expected; skipping"
                        );
                        continue;
                    }
                }
                hmgr::windows::set_user(&key, "")?;
            }
            let path = hmgr::windows::get_user("PATH")?;
            let mut new_paths = vec![];
            for p in path.split(';') {
                let p = p.trim();
                if p.is_empty() {
                    continue;
                }
                if path_to_remove.contains(p) {
                    continue;
                }
                new_paths.push(p)
            }
            let new_path = new_paths.join(";");
            hmgr::windows::set_user("PATH", &new_path)?;
        }

        Ok(())
    }

    #[cu::context("failed to build installed items")]
    pub fn rebuild_items(&mut self, bar: Option<&Arc<cu::ProgressBar>>) -> cu::Result<()> {
        if !self.dirty {
            return Ok(());
        }

        self.items.sort_by_key(|x| std::cmp::Reverse(x.priority));

        let reinvocation_needed = self.env.rebuild(&self.items, self.skip_reinvocation)?;
        if self.link_dirty {
            self.rebuild_links()?;
            self.link_dirty = false;
        }
        if self.shim_dirty {
            self.rebuild_shim(bar)?;
            self.shim_dirty = false;
        }

        let config_path = hmgr::paths::items_config_json();
        cu::fs::write_json_pretty(config_path, &self.items)?;

        if !self.skip_reinvocation && reinvocation_needed {
            hmgr::require_envchange_reinvocation()?;
        }

        self.dirty = false;
        Ok(())
    }

    #[cu::context("failed to build binary links")]
    fn rebuild_links(&mut self) -> cu::Result<()> {
        let bin_root = hmgr::paths::bin_root();
        cu::fs::make_dir(&bin_root)?;
        let mut link_paths = vec![];
        for entry in &self.items {
            let Item::LinkBin(from, to, non_exe) = &entry.item else {
                continue;
            };
            let link_path = bin_root.join(from);
            if link_path.exists() {
                // assume existing file is from linking previously
                continue;
            }
            link_paths.push((link_path, to, non_exe));
        }
        let link_paths2: Vec<(&Path, &Path)> = link_paths
            .iter()
            .map(|(x, y, _)| (x.as_path(), y.as_ref()))
            .collect();

        opfs::hardlink_files(&link_paths2)?;

        #[cfg(not(windows))]
        {
            for (from, _, non_exe) in link_paths {
                if !non_exe {
                    opfs::set_executable(&from)?;
                }
            }
        }
        Ok(())
    }

    #[cu::context("failed to build shims")]
    fn rebuild_shim(&self, bar: Option<&Arc<cu::ProgressBar>>) -> cu::Result<()> {
        let mut shim_config = ShimConfig::default();
        for entry in &self.items {
            use std::collections::btree_map::Entry;
            let Item::ShimBin(name, args) = &entry.item else {
                continue;
            };

            match shim_config.entry(name.to_string()) {
                Entry::Occupied(_) => {
                    cu::bail!("a shim config for '{name}' already exists");
                }
                Entry::Vacant(e) => {
                    e.insert(args.clone());
                }
            }
        }
        let config_path = hmgr::paths::shim_config_json();
        cu::fs::write_json_pretty(&config_path, &shim_config)?;

        hmgr::tools::ensure_unpacked()?;
        let mut shim_path = hmgr::paths::tools_root();
        shim_path.push("shaftim");

        // ensure main.rs exists. This is generated by the build script,
        // however, cargo now requires this file to exists before running anything
        cu::fs::write(shim_path.join("main.rs"), "")?;

        let command = cu::which("cargo")?
            .command()
            // setting current dir in case the current directory the user is in has a
            // rust-toolchain file, which will override the rust toolchain being used
            .current_dir(&shim_path)
            .env("SHAFT_SHIM_BUILD_CONFIG", &config_path)
            .add(cu::args![
                "build",
                "--release",
                "--manifest-path",
                shim_path.join("Cargo.toml")
            ]);
        let command = epkg::cargo::add_platform_build_args(command);
        let (child, bar) = command
            .preset(
                cu::pio::cargo("building shaft shim").configure_spinner(|x| x.parent(bar.cloned())),
            )
            .spawn()?;
        child.wait_nz()?;
        bar.done();
        let mut shim_path = hmgr::paths::tools_root();
        shim_path.extend(["target", "release", bin_name!("shaftim")]);
        let shim_binary = hmgr::paths::shim_binary();
        let shim_binary_old = hmgr::paths::shim_binary_old();
        if shim_binary.exists() {
            // hardlink the old binary, so we can start deleting the old links
            opfs::hardlink_files(&[(&shim_binary_old, &shim_binary)])?;
        }

        // the old binary could be in use, which will not allow us to copy it,
        // but we can remove it because it's hardlinked
        opfs::safe_remove_link(&shim_binary)?;
        cu::fs::copy(&shim_path, &shim_binary)?;

        // create new links
        let bin_root = hmgr::paths::bin_root();
        cu::fs::make_dir(&bin_root)?;
        let mut link_paths = Vec::with_capacity(shim_config.len());
        for name in shim_config.keys() {
            let target = bin_root.join(name);
            link_paths.push(target);
        }
        let link_paths = link_paths
            .iter()
            .map(|x| (x.as_path(), shim_binary.as_path()))
            .collect::<Vec<_>>();
        cu::check!(
            opfs::hardlink_files(&link_paths),
            "failed to create hardlinks for shim binaries"
        )?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemEntry {
    pub package: String,
    pub item: Item,
    /// Higher is applied first
    #[serde(default)]
    pub priority: i32,
}

/// An item is an injection to the installation. Packages
/// register these items on install, and when uninstalled,
/// these items will be automatically cleaned up.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Item {
    /// Setting a user environment variable.
    ///
    /// This corresponds to adding to the init shell profiles (bash_profile)
    /// on non-Windows, and setting user environment registry on Windows
    UserEnvVar(String, String),

    /// Add to user PATH
    ///
    /// This corresponds to adding to the init shell profiles (bash_profile)
    /// on non-Windows, and setting user PATH environment registry on Windows
    UserPath(String),

    /// Set environment for compositor-specific environment
    ///
    /// In linux, compositor environment are usually on top of
    /// shell environments as display manager like SDDM will source
    /// bash_profile.
    #[cfg(target_os = "linux")]
    SessionEnvVar(SessionType, String, String),

    /// Link a binary (in the HOME/bin directory) to a location
    /// in the install directory.
    LinkBin(String, String, bool /* non_executable */),

    /// Create a shim binary that invokes a command.
    ///
    /// This is useful in 2 scenarios:
    /// 1. If the target binary can only be invoked at the installed path
    ///    (because of DLL dependency, usually)
    /// 2. If the target binary or script should be invoked with extra
    ///    arguments.
    ShimBin(String, ShimCommand),

    /// Powershell script added to the init.pwsh script
    Pwsh(String),

    /// Dosbatch script added to the init.cmd script
    Cmd(String),

    /// Bash script added to init.bash script
    ///
    /// Use UserEnvVar or UserPath to modify environment variables and PATHs
    /// to auto apply to all shells
    Bash(String),

    /// Zsh script added to init.zsh script
    ///
    /// Use UserEnvVar or UserPath to modify environment variables and PATHs
    /// to auto apply to all shells
    Zsh(String),
}

impl Item {
    #[inline(always)]
    pub fn user_env_var(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self::UserEnvVar(key.into(), value.into())
    }

    #[inline(always)]
    pub fn user_path(path: impl Into<String>) -> Self {
        Self::UserPath(path.into())
    }

    #[inline(always)]
    #[cfg(target_os = "linux")]
    pub fn session_env(
        compositor: SessionType,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self::SessionEnvVar(compositor, key.into(), value.into())
    }

    #[inline(always)]
    pub fn link_bin(name: impl Into<String>, target: impl Into<String>) -> Self {
        Self::LinkBin(name.into(), target.into(), false)
    }

    #[inline(always)]
    #[cfg(not(windows))]
    pub fn link_non_exe(name: impl Into<String>, target: impl Into<String>) -> Self {
        Self::LinkBin(name.into(), target.into(), true)
    }

    #[inline(always)]
    pub fn shim_bin(name: impl Into<String>, command: ShimCommand) -> Self {
        Self::ShimBin(name.into(), command)
    }

    #[inline(always)]
    pub fn pwsh(script: impl Into<String>) -> Self {
        Self::Pwsh(script.into())
    }

    #[inline(always)]
    pub fn cmd(script: impl Into<String>) -> Self {
        Self::Cmd(script.into())
    }

    #[inline(always)]
    pub fn bash(script: impl Into<String>) -> Self {
        Self::Bash(script.into())
    }

    #[inline(always)]
    pub fn zsh(script: impl Into<String>) -> Self {
        Self::Zsh(script.into())
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionType {
    /// Environment: sourced from ~/.config/hyprland/hyprland.conf
    Hyprland,
}

pub fn build_env_map(items: &[ItemEntry]) -> cu::Result<Vec<(String, String)>> {
    let mut seen_key = BTreeSet::new();
    let mut envs = vec![];
    for entry in items {
        let Item::UserEnvVar(key, value) = &entry.item else {
            continue;
        };
        if key.to_lowercase() == "path" {
            cu::bail!("unexpected: use Item::UserPath to set PATH");
        }
        let key = key.trim();
        if !seen_key.insert(key) {
            cu::bail!("an env config for '{key}' already exists");
        }
        envs.push((key.to_string(), value.trim().to_string()));
    }
    Ok(envs)
}
