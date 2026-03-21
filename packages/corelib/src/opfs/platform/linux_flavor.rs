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
