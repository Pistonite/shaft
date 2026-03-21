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
    let mut arch = cu::env_var("PROCESSOR_ARCHITECTURE")?;
    arch.make_ascii_lowercase();
    match arch.trim() {
        "amd64" => Ok(CpuArch::X64),
        "arm64" => Ok(CpuArch::Arm64),
        other => {
            cu::bail!("unknown processor architecture: {other}");
        }
    }
}
