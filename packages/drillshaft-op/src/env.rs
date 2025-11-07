
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use cu::pre::*;

#[cfg(windows)]
pub fn get_current_user_env() -> cu::Result<()> {}
#[cfg(windows)]
pub fn set_current_user_env() -> cu::Result<()> {}

pub struct EnvChangeReboot {
    path: PathBuf,
    map: BTreeMap<String, String>,
}
impl EnvChangeReboot {
    /// Read env-change-reboot.json at path if exists.
    pub fn new(path: PathBuf) -> cu::Result<Self> {
        if path.exists() {
            let map: BTreeMap<String, String> = json::read(cu::fs::reader(&path)?)?;
            Ok(Self { path, map })
        } else {
            Ok(Self {
                path,
                map: Default::default(),
            })
        }
    }

    /// Add an expectation
    #[inline(always)]
    pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.map.insert(key.into(), value.into());
    }

    #[inline(always)]
    pub fn write_and_bail(self) -> cu::Result<()> {
        cu::fs::write_json_pretty(self.path, &self.map)?;
        cu::hint!(
            "environment variables have changed - please restart the shell/terminal process."
        );
        cu::bail!("please restart the shell/terminal process and execute the command again");
    }

    pub fn check(self) -> cu::Result<()> {
        for (key, value) in self.map {
            let actual = cu::env_var(&key)?;
            cu::ensure!(
                actual == value,
                "env check failed for '{key}': expected '{value}', actual '{actual}'"
            );
        }
        let _ = cu::fs::remove(self.path);
        Ok(())
    }
}
