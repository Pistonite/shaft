use cu::pre::*;
use flate2::read::GzDecoder;
use tar::Archive as TarArchive;

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
    let mut archive = TarArchive::new(GzDecoder::new(TOOLS_TAR_GZ));
    let tools_path = hmgr::paths::tools_root();
    cu::fs::make_dir_empty(&tools_path)?;
    archive.unpack(&tools_path)?;
    let version = opfs::cli_version();
    cu::fs::write(hmgr::paths::tools_version(), version)?;
    Ok(())
}
