//! GNU and LLVM C/C++ Toolchain

use crate::pre::*;

#[rustfmt::skip]
register_binaries!(
    "c++", "gcc", "g++",
    "c++filt", "objdump", "strings", "strip",
    "make"
);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    let v = check_apt!("build-essential");
    let v = v.replace("ubuntu", ".");
    let build_essential = metadata::build_essential_apt::VERSION.replace("ubuntu", ".");
    check_outdated!(&v, &build_essential);
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::apt::install("build-essential", ctx.bar_ref())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    cu::warn!("not uninstalling GCC for your sanity");
    Ok(())
}
