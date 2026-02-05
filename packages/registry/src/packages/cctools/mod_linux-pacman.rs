//! GNU and LLVM C/C++ Toolchain

use crate::pre::*;

// The list is not full, see config.toml
#[rustfmt::skip]
register_binaries!(
    "c++", "gcc", "g++",
    "c++filt", "objdump", "strings", "strip",
    "clang", "clang++", "clang-format", "clang-tidy", "clangd",
    "make", "ninja"
);

pub fn binary_dependencies() -> EnumSet<BinId> {
    enum_set! { BinId::Python }
}

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
    let v = check_pacman!("ninja");
    check_outdated!(&v, metadata[ninja]::VERSION);
    Ok(Verified::UpToDate)
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    epkg::pacman::install("gcc", ctx.bar_ref())?;
    epkg::pacman::install("binutils", ctx.bar_ref())?;
    epkg::pacman::install("gdb", ctx.bar_ref())?;
    epkg::pacman::install("clang", ctx.bar_ref())?;
    epkg::pacman::install("llvm", ctx.bar_ref())?;
    epkg::pacman::install("lldb", ctx.bar_ref())?;
    epkg::pacman::install("cmake", ctx.bar_ref())?;
    epkg::pacman::install("ninja", ctx.bar_ref())?;
    Ok(())
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    epkg::pacman::uninstall("lldb", ctx.bar_ref())?;
    epkg::pacman::uninstall("clang", ctx.bar_ref())?;
    epkg::pacman::uninstall("llvm", ctx.bar_ref())?;
    cu::warn!("not uninstalling GCC and ninja for your sanity");
    Ok(())
}
