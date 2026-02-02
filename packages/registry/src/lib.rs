mod package;
pub use package::*;
mod context;
pub use context::*;
mod macros;
pub(crate) use macros::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[path = "./metadata.gen.rs"]
#[rustfmt::skip]
mod metadata;

pub use _gen::{BinId, PkgId};
#[path = "./package_stub.rs"]
pub(crate) mod _stub;

pub(crate) mod pre {
    #[allow(unused)]
    pub(crate) use crate::{
        BinId, Context, PkgId, Verified, check_bin_in_path, check_bin_in_path_and_shaft,
        check_installed_with_cargo, check_outdated, metadata, register_binaries, test_config,
    };
    #[cfg(target_os = "linux")]
    pub(crate) use crate::{check_installed_pacman_package, check_installed_with_pacman};
    pub(crate) use corelib::hmgr::Item;
    pub(crate) use corelib::hmgr::config::ConfigDef;
    #[allow(unused)]
    pub(crate) use corelib::{
        Version, VersionCache, bin_name, command_output, epkg, hmgr, if_arm, jsexe, opfs,
    };
    pub(crate) use cu::pre::*;
    pub(crate) use enumset::{EnumSet, enum_set};
    pub(crate) use shaftim_build::ShimCommand;
    pub(crate) use std::path::{Path, PathBuf};
}
