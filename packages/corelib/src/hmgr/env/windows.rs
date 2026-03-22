use std::collections::BTreeSet;

use cu::pre::*;

use crate::hmgr;
use crate::hmgr::Item;
use crate::hmgr::item::ItemEntry;

#[derive(Default)]
pub struct Env {
    env_dirty: bool,
    pwsh_dirty: bool,
    cmd_dirty: bool,
}
impl Env {
    pub fn new_dirty() -> Self {
        Self {
            env_dirty: true,
            pwsh_dirty: true,
            cmd_dirty: true,
        }
    }
    pub fn rebuild(&mut self, items: &[ItemEntry], skip_reinvocation: bool) -> cu::Result<bool> {
        let mut reinvocation_needed = false;
        if self.env_dirty {
            reinvocation_needed |= Self::rebuild_registry_env_vars(items, skip_reinvocation)?;
            self.env_dirty = false;
        }
        if self.pwsh_dirty {
            Self::rebuild_pwsh(items)?;
            self.pwsh_dirty = false;
        }
        if self.cmd_dirty {
            Self::rebuild_cmd(items)?;
            self.cmd_dirty = false;
        }
        Ok(reinvocation_needed)
    }

    #[cu::context("failed to build user environment variables")]
    fn rebuild_registry_env_vars(items: &[ItemEntry], skip_reinvocation: bool) -> cu::Result<bool> {
        let envs = hmgr::item::build_env_map(items)?;
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
        let (path, path_changed) = Self::rebuild_path(items)?;
        hmgr::windows::set_user("PATH", &path)?;
        if path_changed {
            cu::debug!("itemmgr: reinvocation because of path: setting path");
            reinvocation_needed = true;
        }
        if reinvocation_needed && !skip_reinvocation {
            hmgr::add_env_assert(envs)?;
        }
        Ok(reinvocation_needed)
    }

    fn rebuild_path(items: &[ItemEntry]) -> cu::Result<(String, bool)> {
        let current_paths = cu::env_var("PATH")?;
        let current_paths: BTreeSet<_> = current_paths
            .split(';')
            .map(|x| x.trim().to_string())
            .collect();

        let mut reinvocation_needed = false;
        let mut controlled_paths = vec![];
        for entry in items {
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
                // we are not adding path asserts here... since
                // it could change (user can add extra paths)
                // it's probably ok to just not assert
                reinvocation_needed = true;
            }
        }
        Ok((out, reinvocation_needed))
    }

    #[cu::context("failed to build powershell profile")]
    fn rebuild_pwsh(items: &[ItemEntry]) -> cu::Result<()> {
        use std::fmt::Write as _;

        let mut out = String::new();
        let _ = writeln!(
            out,
            "# init.ps1; managed by SHAFT, do not edit manually!\n# ==="
        );
        out.push_str(include_str!("init.ps1"));
        out.push('\n');
        let mut current_package = "";
        for entry in items {
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
        Ok(())
    }

    #[cu::context("failed to build dosbatch init")]
    fn rebuild_cmd(items: &[ItemEntry]) -> cu::Result<()> {
        use std::fmt::Write as _;
        let mut out = String::new();
        let _ = writeln!(
            out,
            "@echo off\nrem init.cmd; managed by SHAFT, do not edit manually!\nrem ==="
        );
        out.push_str(include_str!("init.cmd"));
        out.push('\n');

        let mut current_package = "";
        for entry in items {
            let Item::Cmd(script) = &entry.item else {
                continue;
            };
            if entry.package != current_package {
                current_package = &entry.package;
                let _ = writeln!(out, "REM # == {current_package} >>>>>");
            }
            let _ = writeln!(out, "{script}");
        }
        cu::fs::write(hmgr::paths::init_cmd(), &out)?;
        Ok(())
    }

    pub fn on_item_modified(&mut self, entry: &ItemEntry) {
        match &entry.item {
            Item::UserEnvVar(_, _) => self.env_dirty = true,
            Item::UserPath(_) => self.env_dirty = true,
            Item::Pwsh(_) => self.pwsh_dirty = true,
            Item::Cmd(_) => self.cmd_dirty = true,
            Item::LinkBin(_, _, _) => {}
            Item::ShimBin(_, _) => {}
            Item::Bash(_) => {}
            Item::Zsh(_) => {}
        }
    }
}
