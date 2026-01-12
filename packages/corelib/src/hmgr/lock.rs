use std::fs::File;

use cu::pre::*;
use fs2::FileExt;

use crate::hmgr;

pub struct HomeLock(File);
impl Drop for HomeLock {
    fn drop(&mut self) {
        cu::debug!("releasing home directory lock");
        if let Err(e) = self.0.unlock() {
            cu::warn!("failed to unlock home directory: {e:?}");
        }
        let path = hmgr::paths::dot_lock();
        if let Err(e) = cu::fs::remove(path) {
            cu::warn!("failed to remove home directory lock file: {e:?}");
        }
    }
}

/// Lock the program's home directory for exclusive access
pub fn lock() -> cu::Result<HomeLock> {
    let path = hmgr::paths::dot_lock();
    if path.exists() {
        cu::warn!("lock file exists: {}", path.display());
        cu::hint!(
            "there may be another instance of the program running, if not, the program may have crashed earlier and you can manually delete the lock file"
        );
        cu::bail!("lock file exists");
    }
    let file = cu::check!(
        File::create(&path),
        "failed to create lock file at '{}'",
        path.display()
    )?;
    cu::check!(
        file.try_lock_exclusive(),
        "failed to acquire lock on the home directory"
    )?;
    cu::debug!("acquired home directory lock");
    Ok(HomeLock(file))
}
