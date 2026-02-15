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
    let mut ini = opfs::IniFile::open("/etc/sddm.conf.d/default.conf")?;
    let section = ini.section_mut("Autologin");
    section.set("Relogin", "true");
    section.set("Session", "wayland");
    section.set("User", &cfg.autologin.user);
    ini.write()
}
