use cu::pre::*;
use enumset::{EnumSet, EnumSetType};

/// Processor Architecture (supported ones)
#[derive(EnumSetType, Display, DebugCustom)]
#[repr(u8)]
pub enum CpuArch {
    /// x86_64 (amd64)
    #[display("x64")]
    #[debug("x64")]
    X64,
    /// aarch64 (arm64)
    #[display("arm64")]
    #[debug("arm64")]
    Arm64,
}

    impl CpuArch {
        pub const fn all() -> EnumSet<Self> { enumset::enum_set! { Self::X64 | Self::Arm64 } }
        pub const fn none() -> EnumSet<Self> { enumset::enum_set! {} }
    }
    impl From<u8> for CpuArch {
        fn from(value: u8) -> Self {
            match value {
                0 => Self::X64,
                _ => Self::Arm64,
            }
        }
    }
    impl From<CpuArch> for u8 {
        fn from(value: CpuArch) -> Self {
            match value {
                CpuArch::X64 => 0,
                CpuArch::Arm64 => 1,
            }
        }
    }

