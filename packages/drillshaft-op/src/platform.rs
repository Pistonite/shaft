use enumset::{EnumSet, EnumSetType};

/// Platform supported by the package manager
#[derive(EnumSetType)]
#[repr(u8)]
pub enum Platform {
    /// Windows
    Windows,
    /// Arch Linux
    Arch,
}

impl Platform {
    pub const fn all() -> EnumSet<Self> {
        enumset::enum_set! {
            Self::Windows |
            Self::Arch
        }
    }
}

static CURRENT_PLATFORM: cu::Atomic<u8, MaybeUninitPlatform> = cu::Atomic::new_u8(0);
struct MaybeUninitPlatform(Option<Platform>);
impl From<u8> for MaybeUninitPlatform {
    fn from(value: u8) -> Self {
        Self(match value {
            1 => Some(Platform::Windows),
            2 => Some(Platform::Arch),
            _ => None
        })
    }
}
impl From<MaybeUninitPlatform> for u8 {
    fn from(value: MaybeUninitPlatform) -> Self {
        match value.0 {
            None => 0,
            Some(Platform::Windows) => 1,
            Some(Platform::Arch) => 2,
        }
    }
}
impl From<Platform> for MaybeUninitPlatform {
    fn from(value: Platform) -> Self {
        Self(Some(value))
    }
}

/// Initialize the platform variable. Called once at beginning when launching
/// the package manager
#[inline(always)]
pub fn init_platform() -> cu::Result<()> {
    init_platform_impl()
}

#[cfg(windows)]
#[inline(always)]
fn init_platform_impl() -> cu::Result<()> {
    CURRENT_PLATFORM.set(Platform::Windows.into());
    Ok(())
}

#[cfg(not(windows))]
#[inline(always)]
fn init_platform_impl() -> cu::Result<()> {
    use std::path::Path;

    if Path::new("/etc/arch-release").exists() {
        if cu::which("pacman").is_err() {
            cu::bail!("unsupported platform: pacman not available; please fix your system");
        }
        CURRENT_PLATFORM.set(Platform::Arch.into());
        return Ok(());
    }

    cu::bail!("cannot determine the platform of the system");
}

pub fn current_platform() -> Platform {
    // unwrap: None means did not call init
    CURRENT_PLATFORM.get().0.unwrap()
}

#[macro_export]
macro_rules! is_arm {
    () => {
        cfg!(target_arch = "aarch64")
    };
}
