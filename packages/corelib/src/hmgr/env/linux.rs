use cu::pre::*;

use crate::hmgr;
use crate::hmgr::Item;
use crate::hmgr::env::unix;
use crate::hmgr::item::{ItemEntry, SessionType};

#[derive(Default)]
pub struct Env {
    bash_dirty: bool,
    zsh_dirty: bool,
    hyprland_dirty: bool,
}

impl Env {
    pub fn new_dirty() -> Self {
        Self {
            bash_dirty: true,
            zsh_dirty: true,
            hyprland_dirty: true,
        }
    }
    pub fn rebuild(&mut self, items: &[ItemEntry], skip_reinvocation: bool) -> cu::Result<bool> {
        let mut reinvocation_needed = false;
        if self.bash_dirty || self.zsh_dirty {
            let (exports, reinvocation_needed_from_exports) =
                self.rebuild_exports(items, skip_reinvocation)?;
            reinvocation_needed |= reinvocation_needed_from_exports;
            if self.bash_dirty {
                self.rebuild_bash(items, &exports)?;
                self.bash_dirty = false;
            }
            if self.zsh_dirty {
                self.rebuild_zsh(items, &exports)?;
                self.zsh_dirty = false;
            }
        }
        if self.hyprland_dirty {
            reinvocation_needed |= self.rebuild_hyprland(items, skip_reinvocation)?;
            self.hyprland_dirty = false;
        }
        Ok(reinvocation_needed)
    }

    fn rebuild_bash(&self, items: &[ItemEntry], exports: &str) -> cu::Result<()> {
        use std::fmt::Write as _;

        let mut out = String::new();
        let _ = writeln!(
            out,
            r#"# init_profile.bash; managed by SHAFT, do not edit manually!"#
        );
        let _ = writeln!(out, r#"{exports}"#);
        cu::fs::write(hmgr::paths::init_profile_bash(), &out)?;

        out.clear();
        let _ = writeln!(
            out,
            "# init_rc.bash; managed by SHAFT, do not edit manually!\n# ==="
        );
        let mut current_package = "";
        for entry in items {
            let Item::Bash(script) = &entry.item else {
                continue;
            };
            if entry.package != current_package {
                current_package = &entry.package;
                let _ = writeln!(out, "\n\n# == {current_package} >>>>>");
            }
            let _ = writeln!(out, "{script}");
        }
        cu::fs::write(hmgr::paths::init_bash(), &out)?;

        Ok(())
    }

    fn rebuild_zsh(&self, items: &[ItemEntry], exports: &str) -> cu::Result<()> {
        use std::fmt::Write as _;

        let mut out = String::new();
        let _ = writeln!(
            out,
            r#"# init_profile.zsh; managed by SHAFT, do not edit manually!"#
        );
        let _ = writeln!(out, r#"{exports}"#);
        cu::fs::write(hmgr::paths::init_profile_zsh(), &out)?;

        out.clear();
        let _ = writeln!(
            out,
            "# init_rc.zsh; managed by SHAFT, do not edit manually!\n# ==="
        );
        let mut current_package = "";
        for entry in items {
            let Item::Bash(script) = &entry.item else {
                continue;
            };
            if entry.package != current_package {
                current_package = &entry.package;
                let _ = writeln!(out, "\n\n# == {current_package} >>>>>");
            }
            let _ = writeln!(out, "{script}");
        }
        cu::fs::write(hmgr::paths::init_zsh(), &out)?;

        Ok(())
    }

    fn rebuild_exports(
        &self,
        items: &[ItemEntry],
        skip_reinvocation: bool,
    ) -> cu::Result<(String, bool)> {
        use std::fmt::Write as _;

        let mut out = String::new();
        let home = hmgr::paths::home().as_utf8()?;
        let _ = writeln!(
            out,
            "export SHAFT_HOME='{home}'\nexport USERPROFILE=\"$HOME\""
        );
        let envs = hmgr::item::build_env_map(items)?;

        let mut reinvocation_needed = false;
        for (key, value) in &envs {
            let _ = writeln!(out, r#"export {key}='{value}'"#);
            if &cu::env_var(key).unwrap_or_default() != value {
                reinvocation_needed = true;
            }
        }
        if reinvocation_needed && !skip_reinvocation {
            hmgr::add_env_assert(envs.clone())?;
        }
        let (path, path_changed) = unix::rebuild_user_path(items)?;
        if path_changed && !skip_reinvocation {
            hmgr::add_env_assert_once("PATH".to_string(), path.clone())?;
        }
        let _ = writeln!(out, r#"export PATH="{path}""#);
        reinvocation_needed |= path_changed;

        Ok((out, reinvocation_needed))
    }

    fn rebuild_hyprland(&self, items: &[ItemEntry], skip_reinvocation: bool) -> cu::Result<bool> {
        use std::fmt::Write as _;

        let mut reinvocation_needed = false;
        let mut out = String::new();
        let _ = writeln!(
            out,
            "# init_hyprland.conf; managed by SHAFT, do not edit manually!\n# ==="
        );
        let mut current_package = "";
        let mut env_asserts = vec![];
        for entry in items {
            let Item::SessionEnvVar(SessionType::Hyprland, key, value) = &entry.item else {
                continue;
            };
            if key.to_lowercase() == "path" {
                cu::bail!("unexpected: set PATH through Item::UserPath");
            }
            if entry.package != current_package {
                current_package = &entry.package;
                let _ = writeln!(out, "\n\n# == {current_package} >>>>>");
            }
            let _ = writeln!(out, "env = {key},{value}");
            if &cu::env_var(key).unwrap_or_default() != value {
                reinvocation_needed = true;
                env_asserts.push((key.clone(), value.clone()));
            }
        }
        if reinvocation_needed && !skip_reinvocation {
            hmgr::add_env_assert(env_asserts)?;
        }
        cu::fs::write(hmgr::paths::init_hyprland_conf(), &out)?;

        Ok(reinvocation_needed)
    }

    pub fn on_item_modified(&mut self, entry: &ItemEntry) {
        match &entry.item {
            Item::UserEnvVar(_, _) | Item::UserPath(_) => {
                self.bash_dirty = true;
                self.zsh_dirty = true;
            }
            Item::SessionEnvVar(SessionType::Hyprland, _, _) => self.hyprland_dirty = true,
            Item::Bash(_) => self.bash_dirty = true,
            Item::Zsh(_) => self.zsh_dirty = true,
            Item::LinkBin(_, _, _) => {}
            Item::ShimBin(_, _) => {}
            Item::Pwsh(_) => {}
            Item::Cmd(_) => {}
        }
    }
}
