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
    let mut ini = cu::check!(
        opfs::IniFile::open("/etc/sddm.conf.d/default.conf"),
        "failed to open sddm configuration - try rebooting the system"
    )?;
    let section = ini.section_mut("Autologin");
    section.set("Relogin", "true");
    section.set("Session", "wayland");
    section.set("User", &cfg.autologin.user);
    ini.write()?;


    Ok(())
}
