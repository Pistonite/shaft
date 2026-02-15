use crate::pre::*;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SddmConfig {
    autologin: SddmAutoLoginConfig
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SddmAutoLoginConfig {
    /// Username for SDDM auto-login
    user: String,
}
pub fn configure(cfg: &SddmConfig) -> cu::Result<()> {
    opfs::sudo("systemctl", "enabling sddm service")?
        .args(["enable", "sddm.service"])
        .stdoe(cu::lv::P)
        .stdin_null()
        .wait_nz()?;
    let target_config = Path::new("/etc/sddm.conf.d/default.conf");
    let mut current_config = target_config;
    if !current_config.exists() {
        current_config = Path::new("/usr/lib/sddm/sddm.conf.d/default.conf");
    }
    let mut ini = opfs::IniFile::open(current_config)?;
    let section = ini.section_mut("Autologin");
    section.set("Relogin", "true");
    section.set("Session", "wayland");
    section.set("User", &cfg.autologin.user);

    cu::check!(opfs::sudo_write(target_config, ini.to_string()), "failed to save sddm config file")?;

    Ok(())
}
