//! Use C/C++ Toolchain in PATH
use crate::pre::*;

register_binaries!(
    "c++",
    "gcc",
    "g++",
    "c++filt",
    "objdump",
    "strings",
    "strip",
    "clang",
    "clang++",
    "clang-format",
    "clang-tidy",
    "clangd",
    "make",
    "cmake",
    "ninja"
);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_bin_in_path!("cmake");
    check_bin_in_path!("ninja");
    cu::warn!(
        "system-cctools does not check if a working C/C++ Toolchain and tools exists, please check so manually if it does not work"
    );
    Ok(Verified::UpToDate)
}

pub fn install(_: &Context) -> cu::Result<()> {
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    Ok(())
}
