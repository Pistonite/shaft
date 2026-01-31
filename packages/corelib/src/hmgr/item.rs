use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;

use cu::pre::*;

use crate::{bin_name, hmgr, opfs};

#[derive(Default)]
pub struct ItemMgr {
    items: Vec<ItemEntry>,
    skip_reinvocation: bool,
    reinvocation_needed: bool,
    dirty: bool,
    shim_dirty: bool,
    bash_dirty: bool,
    zsh_dirty: bool,
    pwsh_dirty: bool,
}

impl ItemMgr {
    #[cu::context("failed to load installed items")]
    pub fn load() -> cu::Result<Self> {
        let config_path = hmgr::paths::items_config_json();
        let Ok(items) = cu::fs::read_string(config_path) else {
            return Ok(Self {
                items: vec![],
                skip_reinvocation: false,
                reinvocation_needed: false,
                dirty: true,
                shim_dirty: true,
                bash_dirty: true,
                zsh_dirty: true,
                pwsh_dirty: true,
            });
        };
        let items = json::parse(&items)?;
        Ok(Self {
            items,
            skip_reinvocation: false,
            reinvocation_needed: false,
            dirty: false,
            shim_dirty: false,
            bash_dirty: false,
            zsh_dirty: false,
            pwsh_dirty: false,
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
            Item::LinkBin(_, _) => {}
            Item::ShimBin(_, _) => self.shim_dirty = true,
            Item::Pwsh(_) => self.pwsh_dirty = true,
            Item::Bash(_) => self.bash_dirty = true,
            Item::Zsh(_) => self.zsh_dirty = true,
        }
        self.dirty = true;
        self.items.push(entry);
    }

    pub fn remove_package(
        &mut self,
        package: &str,
        bar: Option<&Arc<cu::ProgressBar>>,
    ) -> cu::Result<()> {
        let mut bin_to_remove = vec![];
        let mut _env_to_remove = BTreeMap::new();
        let mut _path_to_remove = BTreeSet::new();
        self.items.retain(|entry| {
            if entry.package != package {
                return true;
            }
            match &entry.item {
                Item::UserEnvVar(k, v) => {
                    _env_to_remove.insert(k.to_string(), v.to_string());
                }
                Item::UserPath(path) => {
                    _path_to_remove.insert(path.to_string());
                }
                Item::LinkBin(bin, _) => bin_to_remove.push(bin.to_string()),
                Item::ShimBin(bin, _) => {
                    bin_to_remove.push(bin.to_string());
                    self.shim_dirty = true;
                }
                Item::Pwsh(_) => self.pwsh_dirty = true,
                Item::Bash(_) => self.bash_dirty = true,
                Item::Zsh(_) => self.zsh_dirty = true,
            }
            self.dirty = true;
            false
        });

        if !bin_to_remove.is_empty() {
            let bar = cu::progress("removing old links")
                .parent(bar.cloned())
                .total(bin_to_remove.len())
                .spawn();
            let bin_root = hmgr::paths::bin_root();
            for bin in bin_to_remove {
                cu::progress!(bar += 1, "{bin}");
                opfs::safe_remove_link(&bin_root.join(bin))?;
            }
        }

        #[cfg(windows)]
        {
            for (key, value) in _env_to_remove {
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
                if _path_to_remove.contains(p) {
                    continue;
                }
                new_paths.push(p)
            }
            let new_path = new_paths.join(";");
            hmgr::windows::set_user("PATH", &new_path)?;
        }

        Ok(())
    }

    pub fn set_need_reinvocation(&mut self) {
        self.reinvocation_needed = true;
    }

    #[cu::context("failed to build installed items")]
    pub fn rebuild_items(&mut self, bar: Option<&Arc<cu::ProgressBar>>) -> cu::Result<()> {
        if !self.dirty {
            return Ok(());
        }

        self.items.sort_by_key(|x| std::cmp::Reverse(x.priority));

        #[cfg(windows)]
        {
            self.rebuild_user_env_vars()?;
        }
        self.rebuild_links()?;

        if cfg!(not(windows)) && self.bash_dirty {
            self.rebuild_bash()?;
        }
        if cfg!(not(windows)) && self.zsh_dirty {
            self.rebuild_zsh()?;
        }
        if cfg!(windows) && self.pwsh_dirty {
            self.rebuild_pwsh()?;
        }
        if self.shim_dirty {
            self.rebuild_shim(bar)?;
        }

        let config_path = hmgr::paths::items_config_json();
        cu::fs::write_json_pretty(config_path, &self.items)?;

        if !self.skip_reinvocation && self.reinvocation_needed {
            hmgr::require_envchange_reinvocation()?;
        }

        self.dirty = false;
        Ok(())
    }

    #[cfg(windows)]
    #[cu::context("failed to build user environment variables")]
    fn rebuild_user_env_vars(&mut self) -> cu::Result<()> {
        let envs = self.build_env_map()?;
        let mut reinvocation_needed = false;
        for (key, value) in &envs {
            if !reinvocation_needed {
                if let Ok(current) = hmgr::windows::get_user_this_session(key) {
                    if &current != value {
                        reinvocation_needed = true;
                        cu::debug!(
                            "itemmgr: reinvocation because of env: '{key}': '{current}'->'{value}'"
                        );
                    }
                } else {
                    cu::debug!(
                        "itemmgr: reinvocation because of env: '{key}': setting new value: '{value}'"
                    );
                    reinvocation_needed = true;
                }
            }
            hmgr::windows::set_user(key, value)?;
        }
        let (path, path_changed) = self.build_user_path()?;
        hmgr::windows::set_user("PATH", &path)?;
        if path_changed {
            cu::debug!("itemmgr: reinvocation because of path: setting path");
            reinvocation_needed = true;
        }
        if reinvocation_needed {
            // we are not adding path asserts here... since
            // it could change (user can add extra paths)
            // it's probably ok to just not assert
            hmgr::add_env_assert(envs)?;
            self.reinvocation_needed = true;
        }
        Ok(())
    }

    #[cu::context("failed to build binary links")]
    fn rebuild_links(&mut self) -> cu::Result<()> {
        let bin_root = hmgr::paths::bin_root();
        cu::fs::make_dir(&bin_root)?;
        let mut link_paths = vec![];
        for entry in &self.items {
            let Item::LinkBin(from, to) = &entry.item else {
                continue;
            };
            let link_path = bin_root.join(from);
            if link_path.exists() {
                // assume existing file is from linking previously
                continue;
            }
            link_paths.push((link_path, to));
        }
        let link_paths: Vec<(&Path, &Path)> = link_paths
            .iter()
            .map(|(x, y)| (x.as_path(), y.as_ref()))
            .collect();

        opfs::hardlink_files(&link_paths)?;
        Ok(())
    }

    #[cu::context("failed to build bash profile")]
    fn rebuild_bash(&mut self) -> cu::Result<()> {
        use std::fmt::Write as _;
        let mut out = include_str!("init.bash").to_string();
        let home = hmgr::paths::home().as_utf8()?;
        let _ = writeln!(out, r#"export SHAFT_HOME='{home}'"#);
        // to be consistent with Windows, we hoist environment to the top
        let envs = self.build_env_map()?;
        let mut reinvocation_needed = false;
        for (key, value) in &envs {
            let _ = writeln!(out, r#"export {key}='{value}'"#);
            if &cu::env_var(key).unwrap_or_default() != value {
                reinvocation_needed = true;
            }
        }

        let (path, path_changed) = self.build_user_path()?;
        let _ = writeln!(out, r#"export PATH="{path}""#);
        let _ = writeln!(out, "# ===");
        let mut current_package = "";
        for entry in &self.items {
            let Item::Bash(script) = &entry.item else {
                continue;
            };
            if entry.package != current_package {
                current_package = &entry.package;
                let _ = writeln!(out, "# == {current_package} >>>>>");
            }
            let _ = writeln!(out, "{script}");
        }

        cu::fs::write(hmgr::paths::init_bash(), out)?;
        if path_changed || reinvocation_needed {
            hmgr::add_env_assert(envs)?;
            self.reinvocation_needed = true;
        }
        self.bash_dirty = false;
        Ok(())
    }

    #[cu::context("failed to build zsh profile")]
    fn rebuild_zsh(&mut self) -> cu::Result<()> {
        use std::fmt::Write as _;
        let mut out = include_str!("init.zsh").to_string();
        let home = hmgr::paths::home().as_utf8()?;
        let _ = writeln!(out, r#"export SHAFT_HOME='{home}'"#);
        // to be consistent with Windows, we hoist environment to the top
        let envs = self.build_env_map()?;
        let mut reinvocation_needed = false;
        for (key, value) in &envs {
            let _ = writeln!(out, r#"export {key}='{value}'"#);
            if &cu::env_var(key).unwrap_or_default() != value {
                reinvocation_needed = true;
            }
        }

        let (path, path_changed) = self.build_user_path()?;
        let _ = writeln!(out, r#"export PATH="{path}""#);
        let _ = writeln!(out, "# ===");
        let mut current_package = "";
        for entry in &self.items {
            let Item::Zsh(script) = &entry.item else {
                continue;
            };
            if entry.package != current_package {
                current_package = &entry.package;
                let _ = writeln!(out, "# == {current_package} >>>>>");
            }
            let _ = writeln!(out, "{script}");
        }

        cu::fs::write(hmgr::paths::init_zsh(), out)?;
        if path_changed || reinvocation_needed {
            hmgr::add_env_assert(envs)?;
            self.reinvocation_needed = true;
        }
        self.zsh_dirty = false;
        Ok(())
    }

    #[cu::context("failed to build powershell profile")]
    fn rebuild_pwsh(&mut self) -> cu::Result<()> {
        use std::fmt::Write as _;
        let mut out = include_str!("init.ps1").to_string();
        let mut current_package = "";
        for entry in &self.items {
            let Item::Pwsh(script) = &entry.item else {
                continue;
            };
            if entry.package != current_package {
                current_package = &entry.package;
                let _ = writeln!(out, "# == {current_package} >>>>>");
            }
            let _ = writeln!(out, "{script}");
        }
        cu::fs::write(hmgr::paths::init_ps1(), &out)?;
        self.pwsh_dirty = false;
        Ok(())
    }

    fn build_env_map(&self) -> cu::Result<Vec<(String, String)>> {
        let mut seen_key = BTreeSet::new();
        let mut envs = vec![];
        for entry in &self.items {
            let Item::UserEnvVar(key, value) = &entry.item else {
                continue;
            };
            if key.to_lowercase() == "path" {
                cu::bail!("please use Item::UserPath to set PATH");
            }
            let key = key.trim();
            if !seen_key.insert(key) {
                cu::bail!("an env config for '{key}' already exists");
            }
            envs.push((key.to_string(), value.trim().to_string()));
        }
        Ok(envs)
    }

    // return the PATH and if reinvocation is needed
    fn build_user_path(&self) -> cu::Result<(String, bool)> {
        let current_paths = cu::env_var("PATH")?;
        let current_paths: BTreeSet<_> = if cfg!(windows) {
            current_paths
                .split(';')
                .map(|x| x.trim().to_string())
                .collect()
        } else {
            current_paths
                .split(':')
                .map(|x| x.trim().to_string())
                .collect()
        };

        let mut reinvocation_needed = false;
        let mut controlled_paths = vec![];
        for entry in &self.items {
            let Item::UserPath(p) = &entry.item else {
                continue;
            };
            controlled_paths.push(p);
            if !current_paths.contains(p) {
                cu::debug!("itemmgr: reinvocation because of path: adding '{p}'");
                reinvocation_needed = true;
            }
        }
        let mut seen = BTreeSet::new();
        let mut out = String::new();

        #[cfg(not(windows))]
        {
            use std::fmt::Write as _;
            // on non-Windows, simply append to existing $PATH in the shell
            let _ = write!(out, "$SHAFT_HOME/bin");
            // latest added path go to the front
            for p in controlled_paths.iter().rev() {
                let p = p.trim();
                if p.is_empty() {
                    continue;
                }
                if seen.insert(p) {
                    let _ = write!(out, ":{p}");
                }
            }
            out.push_str(":$PATH");
        }

        #[cfg(windows)]
        {
            use std::fmt::Write as _;
            // on windows, we need to read the existing paths
            let path = hmgr::windows::get_user_this_session("PATH")?;
            let current_paths: BTreeSet<_> =
                path.split(';').map(|x| x.trim().to_string()).collect();
            // To be safe, we will expand %SHAFT_HOME% on windows
            let home_bin = hmgr::paths::bin_root();
            let home_bin_str = home_bin.as_utf8()?;
            out.push_str(home_bin_str);
            seen.insert(home_bin_str);
            // add the new ones
            // latest added path go to the front
            for p in controlled_paths.iter().rev() {
                let p = p.trim();
                if p.is_empty() {
                    continue;
                }
                if seen.insert(p) {
                    let _ = write!(out, ";{p}");
                    // we want to make sure the current path we are getting
                    // is from the User env var, so it's persistent
                    if !current_paths.contains(p) {
                        cu::debug!(
                            "itemmgr: reinvocation because of path: '{p}' was not from user env"
                        );
                        reinvocation_needed = true;
                    }
                }
            }
            // add the old ones
            for p in path.split(';') {
                let p = p.trim();
                if p.is_empty() {
                    continue;
                }
                if seen.insert(p) {
                    let _ = write!(out, ";{p}");
                }
            }
            if out != hmgr::windows::get_user_this_session("PATH")? {
                reinvocation_needed = true;
            }
        }
        Ok((out, reinvocation_needed))
    }

    #[cu::context("failed to build shims")]
    fn rebuild_shim(&mut self, bar: Option<&Arc<cu::ProgressBar>>) -> cu::Result<()> {
        let mut shim_config = BTreeMap::<String, Vec<String>>::default();
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
        shim_path.push("shim-build");

        let (child, bar) = cu::which("cargo")?
            .command()
            .env("SHAFT_SHIM_BUILD_CONFIG", &config_path)
            .add(cu::args![
                "build",
                "--release",
                "--manifest-path",
                shim_path.join("Cargo.toml")
            ])
            .preset(
                cu::pio::cargo("building shaft shim").configure_spinner(|x| x.parent(bar.cloned())),
            )
            .spawn()?;
        child.wait_nz()?;
        bar.done();
        shim_path.extend(["target", "release"]);
        shim_path.push(bin_name!("shaftim"));
        let shim_binary = hmgr::paths::shim_binary();
        let shim_binary_old = hmgr::paths::shim_binary_old();
        if shim_binary.exists() {
            // hardlink the old binary, so we can start deleting the old links
            opfs::hardlink_files(&[(&shim_binary_old, &shim_binary)])?;
        }

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

        self.shim_dirty = false;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemEntry {
    package: String,
    item: Item,
    /// Higher is applied first
    #[serde(default)]
    priority: i32,
}

/// An item is an injection to the installation. Packages
/// register these items on install, and when uninstalled,
/// these items will be automatically cleaned up.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Item {
    /// Setting a user environment variable.
    ///
    /// This corresponds to adding to the init shell profiles
    /// on non-Windows, and setting user environment registry on Windows
    UserEnvVar(String, String),

    /// Add to user PATH
    ///
    /// This corresponds to adding to the init shell profiles
    /// on non-Windows, and setting user PATH environment registry on Windows
    UserPath(String),

    /// Link a binary (in the HOME/bin directory) to a location
    /// in the install directory.
    LinkBin(String, String),

    /// Create a shim binary that invokes a command.
    ///
    /// This is useful in 2 scenarios:
    /// 1. If the target binary can only be invoked at the installed path
    ///    (because of DLL dependency, usually)
    /// 2. If the target binary or script should be invoked with extra
    ///    arguments.
    ShimBin(String, Vec<String>),

    /// Powershell script added to the init.pwsh script
    Pwsh(String),

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
