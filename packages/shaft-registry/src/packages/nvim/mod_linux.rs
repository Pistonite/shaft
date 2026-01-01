use cu::pre::*;
use enumset::EnumSet;

use crate::pre::*;

metadata_binaries!("vi", "vim", "nvim");

static VERSION: &str = "0.11.5";

pub fn binary_dependencies() -> EnumSet<BinId> {
    enumset::enum_set!(BinId::_7z)
}

pub fn verify(_: &Context) -> cu::Result<Verified> {
    let bin_dir = op::home::bin_dir();
    let binary = match cu::which("nvim") {
        Err(_) => return Ok(Verified::NotInstalled),
        Ok(path) => {
            if path != bin_dir.join("nvim") {
                cu::bail!("binary 'nvim' is installed outside of this tool, please uninstall it first.");
            }
            path
        }
    };
    let (child, stdout) = binary.command()
    .arg("--version")
        .stdout(cu::pio::string())
        .stdie_null()
        .spawn()?;
    child.wait_nz()?;
    let stdout = stdout.join()??;
    let version_line = stdout.lines().next().unwrap_or("");
    let Some(version) = version_line.strip_prefix("NVIM v") else {
        cu::warn!("nvim --version returned unexpected output: {stdout}");
        return Ok(Verified::NotUpToDate);
    };
    if !version.is_version_same_or_higher_than(VERSION) {
        return Ok(Verified::NotUpToDate);
    }
    
    Ok(Verified::UpToDate)
}

pub fn install(_: &Context) -> cu::Result<()> {
    Ok(())
}
pub fn uninstall(_: &Context) -> cu::Result<()> {
    Ok(())
}
