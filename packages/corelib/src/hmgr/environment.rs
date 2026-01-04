use std::collections::BTreeMap;

use cu::pre::*;

use crate::hmgr;

#[cfg(windows)]
pub mod windows {
    pub use win_envedit::*;
}

/// Add environment assert to ensure proper environment the next time the tool is invoked
pub fn add_env_assert<I: IntoIterator<Item = (String, String)>>(iter: I) -> cu::Result<()> {
    let mut envs = load_env_json()?;
    envs.extend(iter);
    save_env_json(&envs);
    Ok(())
}

/// Load and check if the current environment matches assertions in HOME/environment.json
#[inline(always)]
pub fn init_env() -> cu::Result<()> {
    let env = load_env_json()?;
    let mut new_env = BTreeMap::new();
    let mut ok = true;
    for (key, value) in env {
        match cu::env_var(&key) {
            Err(e) => {
                cu::error!("{e:?}");
                cu::warn!("unable to verify env var '{key}' is set properly!");
            }
            Ok(actual) => {
                if value != actual {
                    cu::error!("env var '{key}' is not set properly!");
                    ok = false;
                    new_env.insert(key, value);
                }
            }
        }
    }
    save_env_json(&new_env);
    if !ok {
        cu::error!("some environment variables are not the expected value");
        return require_envchange_reinvocation(false);
    }
    Ok(())
}

/// Error with message for restarting terminal process to refresh environment
#[inline(always)]
pub fn require_envchange_reinvocation(resume: bool) -> cu::Result<()> {
    match (cfg!(windows), resume) {
        (true, true) => {
            cu::bail!(
                "environment has changed, please restart (all) terminal process, then run `shaft resume`."
            );
        }
        (true, false) => {
            cu::bail!("environment has changed, please restart (all) terminal process.");
        }
        (false, true) => {
            cu::bail!(
                "environment has changed, please restart the shell, then run `shaft resume`."
            );
        }
        (false, false) => {
            cu::bail!("environment has changed, please restart the shell.");
        }
    }
}

fn load_env_json() -> cu::Result<BTreeMap<String, String>> {
    match cu::fs::read_string(hmgr::paths::environment_json()) {
        Ok(content) => {
            let map: BTreeMap<String, String> = cu::check!(
                json::parse(&content),
                "failed to parse env mod json, please manually check for corruption in the file"
            )?;
            Ok(map)
        }
        Err(_) => Ok(Default::default()),
    }
}

fn save_env_json(map: &BTreeMap<String, String>) {
    let path = hmgr::paths::environment_json();
    if map.is_empty() {
        let _ = cu::fs::remove(path);
        return;
    }
    if let Err(e) = cu::fs::write_json_pretty(path, map) {
        // if save errored, not much we can do, print a warning
        cu::error!("error while saving env json: {e:?}");
        cu::warn!("failed to save env json, please restart the terminal/shell.");
    }
}
