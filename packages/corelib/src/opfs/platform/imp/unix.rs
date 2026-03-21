use super::{CURRENT_ARCH, CpuArch};
use cu::pre::*;

pub fn init_arch_with_uname(default: CpuArch) -> cu::Result<()> {
    let arch = match get_arch_with_uname() {
        Ok(Some(x)) => {
            x
        }
        Ok(None) => {
            cu::bail!("unknown processor architecture");
        }
        Err(e) => {
            cu::warn!("error while detecting processor architecture: {e:?}");
            cu::hint!("assuming {default}");
            default
        }
    };
    CURRENT_ARCH.set(arch);
    if cfg!(feature = "build-x64") {
        cu::debug!("arch: {arch} +build-x64");
    } else {
        cu::debug!("arch: {arch}");
    }
    Ok(())
}

fn get_arch_with_uname() -> cu::Result<Option<CpuArch>> {
    let (child, stdout) = cu::which("uname")?
        .command()
        .arg("-m")
        .stdout(cu::pio::string())
        .stdin_null()
        .stderr_null()
        .spawn()?;
    child.wait_nz()?;
    let mut stdout = stdout.join()??;
    stdout.make_ascii_lowercase();
    match stdout.trim() {
        "x86_64" => Ok(Some(CpuArch::X64)),
        "aarch64" => Ok(Some(CpuArch::Arm64)),
        other => {
            cu::warn!("unknown architecture from uname -m: {other}");
            Ok(None)
        }
    }
}
