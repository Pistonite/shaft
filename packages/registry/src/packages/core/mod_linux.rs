//! Pseudo package for the package manager itself

mod common;
pub use common::{verify, install, config_location, pre_uninstall, pre_uninstall as uninstall};

crate::register_binaries!("sudo", "cargo", "bash");
