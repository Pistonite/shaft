use std::sync::OnceLock;

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[path = "imp/unix.rs"]
mod imp_unix;

#[cfg(windows)]
#[path = "imp/windows.rs"]
mod imp;

#[cfg(target_os = "macos")]
#[path = "imp/mac.rs"]
mod imp;

#[cfg(target_os = "linux")]
#[path = "imp/linux.rs"]
mod imp;

mod cpu_arch;
pub use cpu_arch::*;
mod linux_flavor;
pub use linux_flavor::*;

#[cfg(target_os = "linux")]
static CURRENT_FLAVOR: cu::Atomic<u8, LinuxFlavor> = cu::Atomic::new_u8(0);
static CURRENT_ARCH: cu::Atomic<u8, CpuArch> = cu::Atomic::new_u8(0);
static VERSION: OnceLock<String> = OnceLock::new();

#[cfg(target_os = "linux")]
pub fn linux_flavor() -> LinuxFlavor {
    CURRENT_FLAVOR.get()
}

pub fn cli_version() -> &'static str {
    VERSION.get().expect("version not initialized")
}

pub fn cpu_arch() -> CpuArch {
    CURRENT_ARCH.get()
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub fn is_arm() -> bool {
    true
}

#[cfg(not(target_arch = "aarch64"))]
#[inline(always)]
pub fn is_arm() -> bool {
    CURRENT_ARCH.get() == CpuArch::Arm64
}

/// Initialize the platform variable. Called once at beginning when launching
/// the package manager
#[inline(always)]
pub fn init(version: &str) -> cu::Result<()> {
    crate::internal::ensure_main_thread()?; // record main thread ID
    let _ = VERSION.set(version.to_string());
    imp::init()
}
