use corelib::hmgr;
use cu::pre::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub windows: WindowsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct WindowsConfig {
    #[serde(default)]
    pub control_personal_shell_folder: bool,
    #[serde(default)]
    pub control_home: bool,
}

#[cu::error_ctx("failed to load config")]
pub fn load_config() -> cu::Result<Config> {
    let path = hmgr::paths::config_toml();
    if !path.exists() {
        cu::fs::write(&path, include_str!("./config.toml"))?;
    }
    let config_content = cu::fs::read_string(&path)?;
    let config = toml::parse::<Config>(&config_content)?;
    Ok(config)
}
