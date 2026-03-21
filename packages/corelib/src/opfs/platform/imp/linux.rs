use super::{CURRENT_FLAVOR, CpuArch, LinuxFlavor};
use std::path::Path;
pub fn init() -> cu::Result<()> {
    init_flavor()?;
    super::imp_unix::init_arch_with_uname(CpuArch::X64)
}

fn init_flavor() -> cu::Result<()> {
    if Path::new("/etc/arch-release").exists() {
        if cu::which("pacman").is_err() {
            cu::bail!("unsupported platform: pacman not available; please fix your system");
        }
        CURRENT_FLAVOR.set(LinuxFlavor::Pacman);
        cu::debug!("linux_flavor: pacman (Arch Linux)");
        return Ok(());
    }

    if cu::which("pacman").is_ok() {
        cu::debug!("linux_flavor: pacman (Unknown)");
        CURRENT_FLAVOR.set(LinuxFlavor::Pacman);
        return Ok(());
    }

    if cu::which("apt").is_ok() {
        cu::debug!("linux_flavor: apt (Unknown)");
        CURRENT_FLAVOR.set(LinuxFlavor::Apt);
        return Ok(());
    }

    cu::bail!("cannot determine the flavor of the linux system");
}
