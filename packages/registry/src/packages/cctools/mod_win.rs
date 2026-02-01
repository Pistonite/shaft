//! Windows C/C++ Toolchain via llvm-mingw
use std::collections::BTreeSet;

use crate::pre::*;

// The list is not full, see config.toml
#[rustfmt::skip]
register_binaries!(
    "c++", "gcc", "g++",
    "c++filt", "objdump", "strings", "strip",
    "clang", "clang++", "clang-format", "clang-tidy", "clangd",
    "make", "cmake", "ninja"
);

mod clang;

pub fn binary_dependencies() -> EnumSet<BinId> {
    enum_set! { BinId::Scalar }
}

pub use clang::verify;

pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file("llvm-mingw.zip", url(), metadata::clang::SHA, ctx.bar())?;
    Ok(())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    ctx.move_install_to_old_if_exists()?;
    let clang_zip = hmgr::paths::download("llvm-mingw.zip", url());
    opfs::unarchive(&clang_zip, ctx.install_dir(), true)?;
    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let install_dir = ctx.install_dir();
    let bin_dir = install_dir.join("bin");
    let bin_dir_str = bin_dir.as_utf8()?;

    let mut link_files = vec![];
    let mut shim_files = vec![];
    let mut shim_rename_files = vec![];
    let mut bash_wrap_files = vec![];
    let mut files = BTreeSet::new();
    let mut will_install_files = BTreeSet::new();
    // DLLs
    for entry in cu::fs::read_dir(&bin_dir)? {
        let entry = entry?;
        let file_name = entry.file_name().into_utf8()?;
        if file_name.ends_with(".dll") {
            if !file_name.starts_with("libpython") {
                link_files.push(file_name.clone());
                will_install_files.insert(file_name.clone());
            }
        }
        files.insert(file_name);
    }

    for executable in [
        "c++",
        "c99",
        "cc",
        "gcc",
        "g++",
        "addr2line",
        "ar",
        "nm",
        "objcopy",
        "ranlib",
        "readelf",
        "size",
        "strings",
        "strip",
        "clang",
        "clang++",
        "clang-format",
        "clang-tidy",
        "clangd",
        "lldb",
    ] {
        let file_name = bin_name!(executable);
        if !files.contains(&file_name) {
            cu::bail!("cannot find {file_name} in llvm-mingw installation!");
        }
        will_install_files.insert(file_name.clone());
        shim_files.push(file_name);
    }
    #[allow(clippy::single_element_loop)]
    for (executable, rename) in [("mingw32-make", "make")] {
        let file_name = bin_name!(executable);
        if !files.contains(&file_name) {
            cu::bail!("cannot find {file_name} in llvm-mingw installation!");
        }
        will_install_files.insert(file_name.clone());
        shim_files.push(file_name.clone());
        shim_rename_files.push((file_name, bin_name!(rename)));
    }
    for llvm_executable in [
        "addr2line",
        "ar",
        "cov",
        "cvtres",
        "cxxfilt",
        "dlltool",
        "lib",
        "ml",
        "nm",
        "objcopy",
        "objdump",
        "pdbutil",
        "ranlib",
        "rc",
        "readelf",
        "readobj",
        "size",
        "strings",
        "symbolizer",
        "windres",
    ] {
        let file_name = format!("llvm-{llvm_executable}.exe");
        if !files.contains(&file_name) {
            cu::bail!("cannot find {file_name} in llvm-mingw installation!");
        }
        will_install_files.insert(file_name.clone());
        shim_files.push(file_name);
    }
    for lldb_executable in ["argdumper", "dap", "instr", "server"] {
        let file_name = format!("lldb-{lldb_executable}.exe");
        if !files.contains(&file_name) {
            cu::bail!("cannot find {file_name} in llvm-mingw installation!");
        }
        will_install_files.insert(file_name.clone());
        shim_files.push(file_name);
    }
    for bash_executable in ["ld", "objdump"] {
        if !files.contains(bash_executable) {
            cu::bail!("cannot find {bash_executable} in llvm-mingw installation!");
        }
        will_install_files.insert(bash_executable.to_owned());
        bash_wrap_files.push(bash_executable.to_owned());
    }

    let config = ctx.load_config_file_or_default(include_str!("config.toml"))?;
    if let Some(link_extra_files) = config
        .get("windows-link-extra-files")
        .and_then(|x| x.as_array())
    {
        for value in link_extra_files {
            let Some(value) = value.as_str() else {
                cu::warn!("ignoring bad file in windows-link-extra-files: {value} (not a string)");
                continue;
            };
            if !files.contains(value) {
                cu::warn!("ignoring non-existing file in windows-link-extra-files: {value}");
                continue;
            }
            if will_install_files.contains(value) {
                cu::warn!(
                    "ignoring file in windows-link-extra-files that is already included: {value}"
                );
                continue;
            }
            will_install_files.insert(value.to_owned());
            link_files.push(value.to_owned());
        }
    }
    if let Some(shim_extra_files) = config
        .get("windows-shim-extra-files")
        .and_then(|x| x.as_array())
    {
        for value in shim_extra_files {
            let Some(value) = value.as_str() else {
                cu::warn!("ignoring bad file in windows-shim-extra-files: {value} (not a string)");
                continue;
            };
            if !files.contains(value) {
                cu::warn!("ignoring non-existing file in windows-shim-extra-files: {value}");
                continue;
            }
            if will_install_files.contains(value) {
                cu::warn!(
                    "ignoring file in windows-shim-extra-files that is already included: {value}"
                );
                continue;
            }
            will_install_files.insert(value.to_owned());
            shim_files.push(value.to_owned());
        }
    }
    if let Some(bash_wrap_extra_files) = config
        .get("windows-bash-wrap-extra-files")
        .and_then(|x| x.as_array())
    {
        for value in bash_wrap_extra_files {
            let Some(value) = value.as_str() else {
                cu::warn!(
                    "ignoring bad file in windows-bash-wrap-extra-files: {value} (not a string)"
                );
                continue;
            };
            if !files.contains(value) {
                cu::warn!("ignoring non-existing file in windows-bash-wrap-extra-files: {value}");
                continue;
            }
            if will_install_files.contains(value) {
                cu::warn!(
                    "ignoring file in windows-bash-wrap-extra-files that is already included: {value}"
                );
                continue;
            }
            will_install_files.insert(value.to_owned());
            bash_wrap_files.push(value.to_owned());
        }
    }

    for file in link_files {
        let to = bin_dir.join(&file).into_utf8()?;
        let from = hmgr::paths::binary(file).into_utf8()?;
        ctx.add_item(Item::link_bin(from, to))?;
    }
    for file in shim_files {
        let to = bin_dir.join(&file).into_utf8()?;
        ctx.add_item(Item::shim_bin(
            file,
            ShimCommand::target_paths(to, [bin_dir_str]),
        ))?;
    }
    for (file, rename) in shim_rename_files {
        let to = bin_dir.join(&file).into_utf8()?;
        ctx.add_item(Item::shim_bin(
            rename,
            ShimCommand::target_paths(to, [bin_dir_str]),
        ))?;
    }
    for file in bash_wrap_files {
        let to = bin_dir.join(&file).into_utf8()?;
        ctx.add_item(Item::shim_bin(
            bin_name!(file),
            ShimCommand::target_bash_paths(to, [bin_dir_str]),
        ))?;
    }

    Ok(())
}

pub fn config_location(ctx: &Context) -> cu::Result<Option<PathBuf>> {
    Ok(Some(ctx.config_file()))
}

fn url() -> String {
    let repo = metadata::clang::REPO;
    let tag = metadata::clang::TAG;
    let arch = if_arm!("aarch64", else "x86_64");
    format!("{repo}/releases/download/{tag}/llvm-mingw-{tag}-ucrt-{arch}.zip")
}
