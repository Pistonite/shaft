use super::{CURRENT_FLAVOR, CpuArch, LinuxFlavor};
pub fn init() -> cu::Result<()> {
    // no more x64 apples after 2023, but there are still x64 ones around
    // use x64 for max compatibility
    super::imp_unix::init_arch_with_uname(CpuArch::X64)
}
