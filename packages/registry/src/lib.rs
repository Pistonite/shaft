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

#[path = "./packages.gen.rs"]
#[rustfmt::skip]
mod _gen;
pub use _gen::{BinId, PkgId};
#[path = "./package_stub.rs"]
pub(crate) mod _stub;

pub(crate) mod pre {
    pub(crate) use crate::{
        BinId, Context, Package, PkgId, Verified, check_bin_in_path,
        register_binaries,
    };
    #[cfg(target_os = "linux")]
    pub(crate) use crate::check_installed_with_pacman;
    pub(crate) use corelib::{Version, command_output, epkg, opfs};
    pub(crate) use cu::pre::*;
}
