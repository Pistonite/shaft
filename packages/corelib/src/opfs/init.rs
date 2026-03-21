use std::sync::OnceLock;

use cu::pre::*;
use enumset::{EnumSet, EnumSetType};




#[cfg(target_os = "linux")]
mod imp {
    use super::{CURRENT_FLAVOR, LinuxFlavor};
    use std::path::Path;

    pub fn init() -> cu::Result<()> {
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
}

#[cfg(windows)]
mod imp {
    use super::*;
    pub fn init() -> cu::Result<()> {
        cu::env_var("PROCESSOR_ARCHITECTURE")
        Ok(())
    }
    fn get_arch() -> cu::Result<CpuArch> {
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use super::*;
    pub fn init() -> cu::Result<()> {
        Ok(())
    }
}

