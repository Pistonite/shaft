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
binary_dependencies!(Python);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    let v = check_pacman!("gcc");
    let v = v.split_once('+').map(|x| x.0).unwrap_or(&v);
    check_outdated!(v, metadata[gnucc::gcc]::VERSION);

    let v = check_pacman!("binutils");
    let v = v.split_once('+').map(|x| x.0).unwrap_or(&v);
    check_outdated!(v, metadata[gnucc::binutils]::VERSION);

    let v = check_pacman!("gdb");
    check_outdated!(&v, metadata[gnucc::gdb]::VERSION);
    let v = check_pacman!("clang");
    check_outdated!(&v, metadata[clang]::LLVM_VERSION);
    let v = check_pacman!("llvm");
    check_outdated!(&v, metadata[clang]::LLVM_VERSION);
    let v = check_pacman!("lldb");
    check_outdated!(&v, metadata[clang]::LLVM_VERSION);
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::pacman::install_many(
        &["gcc", "binutils", "gdb", "clang", "llvm", "lldb"],
        "[cctools] installing c-compiler tools",
        ctx.bar_ref(),
    )?;
    Ok(())
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    epkg::pacman::uninstall("lldb", ctx.bar_ref())?;
    epkg::pacman::uninstall("clang", ctx.bar_ref())?;
    epkg::pacman::uninstall("llvm", ctx.bar_ref())?;
    cu::warn!("not uninstalling GCC for your sanity");
    Ok(())
}
