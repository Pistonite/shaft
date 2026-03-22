use std::collections::BTreeMap;

use cu::pre::*;

use crate::hmgr;

#[cfg(windows)]
pub mod windows {
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    pub use win_envedit::get_user;
    static USER_THIS_SESSION: Mutex<BTreeMap<String, String>> = Mutex::new(BTreeMap::new());
    /// Get user environment variable at the start of the session,
    /// before any set calls
    #[inline(always)]
    pub fn get_user_this_session(key: impl AsRef<str>) -> cu::Result<String> {
        get_user_this_session_impl(key.as_ref())
    }
    fn get_user_this_session_impl(key: &str) -> cu::Result<String> {
        let this_session = USER_THIS_SESSION.lock().expect("session env lock failed");
        if let Some(value) = this_session.get(key) {
            return Ok(value.to_string());
        }
        // note there could still be a time of read time of use race condition here
        win_envedit::get_user(key)
    }

    /// Set user environment variable
    #[inline(always)]
    pub fn set_user(key: impl AsRef<str>, value: impl AsRef<str>) -> cu::Result<()> {
        set_user_impl(key.as_ref(), value.as_ref())
    }
    fn set_user_impl(key: &str, value: &str) -> cu::Result<()> {
        let mut this_session = USER_THIS_SESSION.lock().expect("session env lock failed");
        if this_session.contains_key(key) {
            drop(this_session);
            win_envedit::set_user(key, value)?;
            return Ok(());
        }
        let old_value = win_envedit::get_user(key)?;
        this_session.insert(key.to_string(), old_value);
        win_envedit::set_user(key, value)
    }
}

/// Add environment assert to ensure proper environment the next time the tool is invoked
pub fn add_env_assert<I: IntoIterator<Item = (String, String)>>(iter: I) -> cu::Result<()> {
    let mut envs = load_env_json()?;
    envs.extend(iter);
    save_env_json(&envs);
    Ok(())
}

pub fn add_env_assert_once(key: String, value: String) -> cu::Result<()> {
    let mut envs = load_env_json()?;
    envs.insert(key, value);
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
        return require_envchange_reinvocation();
    }
    Ok(())
}

/// Error with message for restarting terminal process to refresh environment
#[inline(always)]
pub fn require_envchange_reinvocation() -> cu::Result<()> {
    if cfg!(windows) {
        cu::bail!(
            "environment has changed, please restart (all) terminal process, then rerun the command"
        );
    } else {
        cu::bail!("environment has changed, please restart the session, then rerun the command");
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
