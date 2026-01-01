use cu::pre::*;
use enumset::{EnumSet, EnumSetType};

/// Linux package manager flavor
#[derive(EnumSetType, Display, DebugCustom)]
#[repr(u8)]
pub enum LinuxFlavor {
    /// Pacman (Arch Linux, etc)
    #[display("pacman")]
    #[debug("pacman")]
    Pacman,
    /// Apt (Ubuntu, etc)
    #[display("apt")]
    #[debug("apt")]
    Apt,
}

impl LinuxFlavor {
    /// Get a set for all of the flavors
    pub const fn all() -> EnumSet<Self> {
        enumset::enum_set! {
            Self::Pacman |
            Self::Apt
        }
    }
    /// Get a set for none of the flavors
    pub const fn none() -> EnumSet<Self> {
        enumset::enum_set! {}
    }
}

#[cfg(target_os = "linux")]
static CURRENT_FLAVOR: cu::Atomic<u8, LinuxFlavor> = cu::Atomic::new_u8(0);
impl From<u8> for LinuxFlavor {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Pacman,
            _ => Self::Apt,
        }
    }
}
impl From<LinuxFlavor> for u8 {
    fn from(value: LinuxFlavor) -> Self {
        match value {
            LinuxFlavor::Pacman => 0,
            LinuxFlavor::Apt => 1,
        }
    }
}

/// Initialize the platform variable. Called once at beginning when launching
/// the package manager
#[cfg(target_os = "windows")]
#[inline(always)]
pub fn init_platform() -> cu::Result<()> {
    Ok(())
}

/// Initialize the platform variable. Called once at beginning when launching
/// the package manager
#[cfg(target_os = "linux")]
#[inline(always)]
pub fn init_platform() -> cu::Result<()> {
    use std::path::Path;

    if Path::new("/etc/arch-release").exists() {
        if cu::which("pacman").is_err() {
            cu::bail!("unsupported platform: pacman not available; please fix your system");
        }
        CURRENT_FLAVOR.set(LinuxFlavor::Pacman);
        cu::debug!("found pacman - arch linux");
        return Ok(());
    }

    if cu::which("pacman").is_ok() {
        cu::debug!("found pacman - assuming using pacman as package manager");
        CURRENT_FLAVOR.set(LinuxFlavor::Pacman);
        return Ok(());
    }

    if cu::which("apt").is_ok() {
        cu::debug!("found apt - assuming using apt as package manager");
        CURRENT_FLAVOR.set(LinuxFlavor::Apt);
        return Ok(());
    }

    cu::bail!("cannot determine the platform of the system");
}

/// Initialize the platform variable. Called once at beginning when launching
/// the package manager
#[cfg(target_os = "macos")]
#[inline(always)]
pub fn init_platform() -> cu::Result<()> {
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn linux_flavor() -> LinuxFlavor {
    CURRENT_FLAVOR.get()
}

#[macro_export]
macro_rules! is_arm {
    () => {
        cfg!(target_arch = "aarch64")
    };
}
