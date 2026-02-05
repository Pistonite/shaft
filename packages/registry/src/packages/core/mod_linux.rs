//! Pseudo package for the package manager itself

use crate::pre::*;

mod common;
pub use common::{config_location, install, pre_uninstall, pre_uninstall as uninstall, verify};

register_binaries!("sudo", "cargo", "bash");
