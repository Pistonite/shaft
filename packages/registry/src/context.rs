use std::path::{Path, PathBuf};

use crate::PkgId;

use corelib::hmgr;

/// Context passed to package functions
pub struct Context {
    /// The id of the package being operated on
    pub pkg: PkgId,
}
impl Context {
    pub fn package_name(&self) -> &'static str {
        self.pkg.to_str()
    }
    pub fn temp_dir(&self) -> PathBuf {
        hmgr::paths::temp_dir(self.package_name())
    }
    // pub fn install_dir(&self) -> PathBuf {
    //     hmgr::paths::install_dir(self.package_name())
    // }
    pub fn check_bin_location(&self, binary: &str, expected: &Path) -> cu::Result<()> {
        let actual = cu::which(binary)?;
        cu::ensure!(
            expected == actual,
            "expected location: '{}', actual location: '{}'",
            expected.display(),
            actual.display()
        );
        Ok(())
    }
}
