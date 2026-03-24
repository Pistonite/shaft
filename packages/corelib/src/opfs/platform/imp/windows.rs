use super::{CURRENT_ARCH, CpuArch};

pub fn init() -> cu::Result<()> {
    let arch = get_arch()?;
    CURRENT_ARCH.set(arch);
    if cfg!(feature = "build-x64") {
        cu::debug!("arch: {arch} +build-x64");
    } else {
        cu::debug!("arch: {arch}");
    }
    Ok(())
}
fn get_arch() -> cu::Result<CpuArch> {
    use windows::Win32::System::SystemInformation::{
        IMAGE_FILE_MACHINE, IMAGE_FILE_MACHINE_AMD64, IMAGE_FILE_MACHINE_ARM64,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, IsWow64Process2};
    let mut process_machine = IMAGE_FILE_MACHINE::default();
    let mut native_machine = IMAGE_FILE_MACHINE::default();
    unsafe {
        IsWow64Process2(
            GetCurrentProcess(),
            &mut process_machine,
            Some(&mut native_machine),
        )?;
    }
    match native_machine {
        IMAGE_FILE_MACHINE_AMD64 => Ok(CpuArch::X64),
        IMAGE_FILE_MACHINE_ARM64 => Ok(CpuArch::Arm64),
        other => {
            cu::bail!("unknown native machine type: {:#06x}", other.0);
        }
    }
}
