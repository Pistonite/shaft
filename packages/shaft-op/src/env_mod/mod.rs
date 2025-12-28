
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use cu::pre::*;

use crate::home;

#[cfg(windows)]
pub mod windows;


pub fn add_assert<I: IntoIterator<Item=(String,String)>>(iter: I) -> cu::Result<()> 
{
    let mut envs = load_env_json()?;
    envs.extend(iter);
    cu::fs::write_json_pretty(home::env_json(), &envs)?;
    Ok(())
}

#[cfg(windows)]
pub fn set_current_user_env(key: &str, value: &str) -> cu::Result<()> {}

pub fn require_reinvocation() -> cu::Result<()> {
    todo!()
}

fn load_env_json() -> cu::Result<BTreeMap<String, String>> {
    match cu::fs::read_string(home::env_json()) {
        Ok(content) => {
            let map: BTreeMap<String, String> = cu::check!(json::parse(&content), "failed to parse env mod json, please manually check for corruption in the file")?;
            Ok(map)
        }
        Err(_) => {
            Ok(Default::default())
        }
    }
}

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
