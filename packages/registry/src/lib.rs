mod package;
pub use package::*;
mod context;
pub use context::*;
mod macros;
pub(crate) use macros::*;

pub enum Verified {
    UpToDate,
    NotUpToDate,
    NotInstalled,
}
impl Verified {
    pub const fn is_installed(installed: bool) -> Self {
        if installed {
            Self::UpToDate
        } else {
            Self::NotInstalled
        }
    }
    pub const fn is_uptodate(uptodate: bool) -> Self {
        if uptodate {
            Self::UpToDate
        } else {
            Self::NotUpToDate
        }
    }
}

#[path = "./packages.gen.rs"]
#[rustfmt::skip]
mod _gen;
pub use _gen::{BinId, PkgId};
#[path = "./package_stub.rs"]
pub(crate) mod _stub;

pub(crate) mod pre {
    #[cfg(target_os = "linux")]
    pub(crate) use crate::check_installed_with_pacman;
    pub(crate) use crate::{
        BinId, Context, Package, PkgId, Verified, check_bin_in_path, check_bin_in_path_and_shaft,
        register_binaries,
    };
    pub(crate) use corelib::{Version, bin_name, command_output, epkg, hmgr, if_arm, opfs};
    pub(crate) use cu::pre::*;
    pub(crate) use enumset::{EnumSet, enum_set};
    pub(crate) use std::path::{Path, PathBuf};
}
