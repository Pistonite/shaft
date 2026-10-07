use std::sync::Mutex;

use cu::pre::*;

use crate::hmgr;
use crate::hmgr::config::ConfigDef;

static CONFIG_DEF: ConfigDef<Config> = ConfigDef::new(
    include_str!("./config.toml"),
    // v1: convert unversioned config to template format + add cargo
    &[""],
);

/// Global config, loaded by [`init_config`]
static CONFIG: Mutex<Option<Config>> = Mutex::new(None);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub windows: WindowsConfig,
    pub cargo: CargoConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct WindowsConfig {
    #[serde(default)]
    pub control_home: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CargoConfig {
    /// Allow cargo-binstall to fallback to compiling from source
    #[serde(default)]
    pub binstall_fallback: bool,
}

/// Load the config file (migrating if needed) into the global config
#[cu::context("failed to load config")]
pub fn init_config() -> cu::Result<()> {
    let config = CONFIG_DEF.load(hmgr::paths::config_toml())?;
    let Ok(mut global) = CONFIG.lock() else {
        cu::bail!("failed to lock global config");
    };
    *global = Some(config);
    Ok(())
}

/// Get the global config. Fails if [`init_config`] has not been called
pub fn get_config() -> cu::Result<Config> {
    let Ok(global) = CONFIG.lock() else {
        cu::bail!("failed to lock global config");
    };
    cu::check!(global.as_ref().cloned(), "config not initialized")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn default_config_is_valid() -> cu::Result<()> {
        CONFIG_DEF.load_default()?;
        Ok(())
    }
}
