//! GNU and LLVM C/C++ Toolchain

use crate::pre::*;

// The list is not full, see config.toml
#[rustfmt::skip]
register_binaries!(
    "c++", "gcc", "g++",
    "c++filt", "objdump", "strings", "strip",
    "clang", "clang++", "clang-format", "clang-tidy", "clangd",
    "make"
);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    let v = check_apt!("build-essential");
    let v = v.replace("ubuntu", ".");
    let build_essential = metadata::build_essential_apt::VERSION.replace("ubuntu", ".");
    check_outdated!(&v, &build_essential);

    let v = check_apt!("clang");
    check_outdated!(&v, metadata[llvm_apt::clang]::VERSION);
    let v = check_apt!("llvm");
    check_outdated!(&v, metadata[llvm_apt::llvm]::VERSION);
    let v = check_apt!("lldb");
    check_outdated!(&v, metadata[llvm_apt::lldb]::VERSION);
    let v = check_apt!("clangd");
    check_outdated!(&v, metadata[llvm_apt::clangd]::VERSION);
    let v = check_apt!("clang-format");
    check_outdated!(&v, metadata[llvm_apt::clang_format]::VERSION);
    let v = check_apt!("clang-tidy");
    check_outdated!(&v, metadata[llvm_apt::clang_tidy]::VERSION);
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::apt::install_many(
        &[
            "build-essential",
            "clang",
            "llvm",
            "lldb",
            "clangd",
            "clang-format",
            "clang-tidy",
        ],
        "[cctools] installing c-compiler tools",
        ctx.bar_ref(),
    )
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    epkg::apt::uninstall("clang-tidy", ctx.bar_ref())?;
    epkg::apt::uninstall("clang-format", ctx.bar_ref())?;
    epkg::apt::uninstall("clangd", ctx.bar_ref())?;
    epkg::apt::uninstall("lldb", ctx.bar_ref())?;
    epkg::apt::uninstall("clang", ctx.bar_ref())?;
    epkg::apt::uninstall("llvm", ctx.bar_ref())?;
    cu::warn!("not uninstalling GCC for your sanity");
    Ok(())
}
