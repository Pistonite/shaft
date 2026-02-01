use cu::pre::*;

use crate::{hmgr, opfs};

static TOOLS_TAR_GZ: &[u8] = include_bytes!("./tools.tar.gz");

/// Ensure the tools directory is unpacked and up to date
pub fn ensure_unpacked() -> cu::Result<()> {
    let version = opfs::cli_version();
    let version_path = hmgr::paths::tools_version();
    let need_unpack = match cu::fs::read_string(version_path) {
        Err(_) => true,
        Ok(x) => x != version,
    };
    if need_unpack {
        cu::check!(do_unpack(), "failed to unpack tools")?;
    }
    Ok(())
}

fn do_unpack() -> cu::Result<()> {
    cu::info!("unpacking tools...");
    let tools_path = hmgr::paths::tools_root();
    cu::check!(
        opfs::untargz_bytes(TOOLS_TAR_GZ, &tools_path, true /* clean */),
        "failed to unpack tools"
    )?;
    let version = opfs::cli_version();
    cu::fs::write(hmgr::paths::tools_version(), version)?;
    Ok(())
}
