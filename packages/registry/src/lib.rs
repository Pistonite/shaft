#[path = "./metadata.gen.rs"]
#[rustfmt::skip]
mod metadata;
pub use metadata::core::CONFIG_VERSION as CORE_VERSION;

#[path = "./packages.gen.rs"]
#[rustfmt::skip]
mod _gen;
pub use _gen::{BinId, PkgId};

mod util;
pub use util::*;

pub(crate) mod pre {
    pub(crate) use crate::macros::*;
    #[allow(unused)]
    pub(crate) use crate::{BinId, Context, PkgId, Verified, metadata};
    pub(crate) use corelib::hmgr::Item;
    pub(crate) use corelib::hmgr::config::ConfigDef;
    #[allow(unused)]
    pub(crate) use corelib::{
        Version, VersionCache, bin_name, command_output, epkg, hmgr, jsexe, opfs,
    };
    pub(crate) use cu::pre::*;
    pub(crate) use enumset::{EnumSet, enum_set};
    pub(crate) use shaftim_build::ShimCommand;
    pub(crate) use std::path::{Path, PathBuf};
}
