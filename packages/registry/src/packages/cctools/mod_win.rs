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

#[rustfmt::skip]
static GNU_CC_BINUTILS_SHIM: &[&str] = &[
    "c++", "c99", "cc", "gcc", "g++",
    "addr2line", "ar", /*c++filt is linked*/ /*ld is bash*/
    "nm", "objcopy" /*objdump is bash*/, "ranlib",
    "readelf", "size", "strings", "strip"
];
#[rustfmt::skip]
static GNU_CC_BINUTILS_SHIM_RENAME: &[(&str, &str)] = &[
    ("c++filt", "llvm-cxxfilt"),
    ("make", "mingw32-make"), // not really binutils but anyway
];
#[rustfmt::skip]
static GNU_CC_BINUTILS_BASH_WRAP: &[&str] = &[ "ld","objdump" ];
#[rustfmt::skip]
static CLANG_LLVM_SHIM: &[&str] = &[
    "amdgpu-arch", "c-index-test", "diagtool", "find-all-symbols",
    "modularize", "nvptx-arch", "offload-arch", "pp-trace",
    "bugpoint", "dsymutil", "opt", "reduce-chunk-list",
    "sancov", "sanstats", "verify-uselistorder", "wasm-ld"
];
//clang*, llvm*, lldb* are also included
//*lld* are also included
#[rustfmt::skip]
static CLANG_LLVM_PYTHON_WRAP: &[&str] = &[
    "analyze-build", "git-clang-format", "hmaptool", "intercept-build",
    "run-clang-tidy",
];
#[rustfmt::skip]
static CLANG_LLVM_LINK_DLL: &[&str] = &[
    "libclang"
];

mod clang;

pub fn binary_dependencies() -> EnumSet<BinId> {
    enum_set! { BinId::Scalar | BinId::Python }
}
pub fn verify(ctx: &Context) -> cu::Result<Verified> {
    let v = clang::verify(ctx)?;
    if v != Verified::UpToDate {
        return Ok(v);
    }

    check_bin_in_path_and_shaft!("cmake", "system-cctools");
    let v = command_output!("cmake", ["--version"]);
    let mut v = v.split_whitespace();
    if v.next() != Some("cmake") {
        cu::warn!("failed to parse cmake version");
        return Ok(Verified::NotUpToDate);
    }
    if v.next() != Some("version") {
        cu::warn!("failed to parse cmake version");
        return Ok(Verified::NotUpToDate);
    }
    let Some(v) = v.next() else {
        cu::warn!("failed to parse cmake version");
        return Ok(Verified::NotUpToDate);
    };
    check_outdated!(v, metadata::cmake::VERSION);

    check_bin_in_path_and_shaft!("ninja", "system-cctools");
    let v = command_output!("ninja", ["--version"]);
    check_outdated!(&v, metadata::ninja::VERSION);

    Ok(Verified::UpToDate)
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file("llvm.txz", llvm_url(), metadata::clang::SHA, ctx.bar())?;
    hmgr::download_file(
        "llvm-mingw.zip",
        llvm_mingw_url(),
        metadata::llvm_mingw::SHA,
        ctx.bar(),
    )?;
    hmgr::download_file("cmake.zip", cmake_url(), metadata::cmake::SHA, ctx.bar())?;
    hmgr::download_file("ninja.zip", ninja_url(), metadata::ninja::SHA, ctx.bar())?;
    Ok(())
}

fn llvm_url() -> String {
    let repo = metadata::clang::REPO;
    let version = metadata::clang::LLVM_VERSION;
    let artifact = llvm_release_name();
    format!("{repo}/releases/download/llvmorg-{version}/{artifact}.tar.xz")
}

fn llvm_release_name() -> String {
    let version = metadata::clang::LLVM_VERSION;
    let arch = if_arm!("aarch64", else "x86_64");
    format!("clang+llvm-{version}-{arch}-pc-windows-msvc")
}

fn llvm_mingw_url() -> String {
    let repo = metadata::llvm_mingw::REPO;
    let tag = metadata::llvm_mingw::TAG;
    let arch = if_arm!("aarch64", else "x86_64");
    format!("{repo}/releases/download/{tag}/llvm-mingw-{tag}-ucrt-{arch}.zip")
}

fn cmake_url() -> String {
    let repo = metadata::cmake::REPO;
    let version = metadata::cmake::VERSION;
    let arch = if_arm!("arm64", else "x86_64");
    format!("{repo}/releases/download/v{version}/cmake-{version}-windows-{arch}.zip")
}

fn ninja_url() -> String {
    let repo = metadata::ninja::REPO;
    let version = metadata::ninja::VERSION;
    let arch = if_arm!("winarm64", else "win");
    format!("{repo}/releases/download/v{version}/ninja-{arch}.zip")
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    ctx.move_install_to_old_if_exists()?;
    let install_dir = ctx.install_dir();

    {
        let bar = cu::progress("unpacking llvm")
            .keep(true)
            .parent(ctx.bar())
            .spawn();
        let llvm_dir = install_dir.join("llvm");
        if llvm_dir.exists() {
            cu::fs::rec_remove(&llvm_dir)?;
        }
        let clang_zip = hmgr::paths::download("llvm.txz", llvm_url());
        opfs::unarchive(&clang_zip, &install_dir, true)?;
        let dir_name = install_dir.join(llvm_release_name());
        cu::check!(
            std::fs::rename(dir_name, llvm_dir),
            "failed to rename directory when unpacking llvm"
        )?;
        bar.done();
    }
    {
        let bar = cu::progress("unpacking llvm-mingw")
            .keep(true)
            .parent(ctx.bar())
            .spawn();
        let llvm_dir = install_dir.join("llvm-mingw");
        let clang_zip = hmgr::paths::download("llvm-mingw.zip", llvm_mingw_url());
        opfs::unarchive(&clang_zip, llvm_dir, true)?;
        bar.done();
    }
    {
        let bar = cu::progress("unpacking cmake")
            .keep(true)
            .parent(ctx.bar())
            .spawn();
        let cmake_dir = install_dir.join("cmake");
        let cmake_zip = hmgr::paths::download("cmake.zip", cmake_url());
        opfs::unarchive(&cmake_zip, cmake_dir, true)?;
        bar.done();
    }
    {
        let bar = cu::progress("unpacking ninja")
            .keep(true)
            .parent(ctx.bar())
            .spawn();
        let ninja_dir = install_dir.join("ninja");
        let ninja_zip = hmgr::paths::download("ninja.zip", ninja_url());
        opfs::unarchive(&ninja_zip, ninja_dir, true)?;
        bar.done();
    }

    Ok(())
}

pub fn uninstall(_: &Context) -> cu::Result<()> {
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    configure_cmake(ctx)?;
    configure_ninja(ctx)?;
    let bar = cu::progress("linking toolchain executables")
        .parent(ctx.bar())
        .spawn();
    let install_dir = ctx.install_dir();
    let llvm_msvc_dir = install_dir.join("llvm\\bin");
    let llvm_mingw_dir = install_dir.join("llvm-mingw\\bin");
    let llvm_msvc_dir_str = llvm_msvc_dir.as_utf8()?;
    let llvm_mingw_dir_str = llvm_mingw_dir.as_utf8()?;

    let config = ctx.load_config(CONFIG)?;

    // these are all relative to install_dir
    let mut link_files = vec![];
    let mut shim_files_gnu = vec![];
    let mut shim_files_msvc = vec![];
    let mut shim_rename_files_gnu = vec![];
    let shim_rename_files_msvc: Vec<(String, String)> = vec![];
    let mut bash_wrap_files_gnu = vec![];
    let mut bash_wrap_files_msvc = vec![];
    let mut python_wrap_files_gnu = vec![];
    let mut python_wrap_files_msvc = vec![];

    // all file names (relative to install_dir)
    let mut source_files = BTreeSet::new();
    // "seen" names in <HOME>/bin, used to detect duplicates
    let mut target_names = BTreeSet::new();

    // collect all file names
    for entry in cu::fs::read_dir(&llvm_msvc_dir)? {
        let entry = entry?;
        let file_name = entry.file_name().into_utf8()?;
        let rel_file_name = format!("llvm\\bin\\{file_name}");
        if file_name.ends_with(".exe")
            && (file_name.starts_with("clang")
                || file_name.starts_with("llvm")
                || file_name.starts_with("lldb")
                || file_name.contains("lld"))
        {
            shim_files_msvc.push(rel_file_name.clone());
            target_names.insert(file_name);
        }
        source_files.insert(rel_file_name);
    }
    for entry in cu::fs::read_dir(&llvm_mingw_dir)? {
        let entry = entry?;
        let file_name = entry.file_name().into_utf8()?;
        source_files.insert(format!("llvm-mingw\\bin\\{file_name}"));
    }

    // GNU CC + Binutils
    for name in GNU_CC_BINUTILS_SHIM {
        shim_files_gnu.push(format!("llvm-mingw\\bin\\{name}.exe"));
        if !target_names.insert(format!("{name}.exe")) {
            cu::bail!("duplicate target name: {name}");
        }
    }
    for (target, source) in GNU_CC_BINUTILS_SHIM_RENAME {
        shim_rename_files_gnu.push((
            format!("{target}.exe"),
            format!("llvm-mingw\\bin\\{source}.exe"),
        ));
        if !target_names.insert(format!("{target}.exe")) {
            cu::bail!("duplicate target name: {target}");
        }
    }
    for name in GNU_CC_BINUTILS_BASH_WRAP {
        bash_wrap_files_gnu.push(format!("llvm-mingw\\bin\\{name}"));
        if !target_names.insert(format!("{name}.exe")) {
            cu::bail!("duplicate target name: {name}");
        }
    }
    for name in CLANG_LLVM_SHIM {
        shim_files_msvc.push(format!("llvm\\bin\\{name}.exe"));
        if !target_names.insert(format!("{name}.exe")) {
            cu::bail!("duplicate target name: {name}");
        }
    }
    for name in CLANG_LLVM_PYTHON_WRAP {
        python_wrap_files_msvc.push(format!("llvm\\bin\\{name}"));
        if !target_names.insert(format!("{name}.exe")) {
            cu::bail!("duplicate target name: {name}");
        }
    }
    for name in CLANG_LLVM_LINK_DLL {
        link_files.push(format!("llvm\\bin\\{name}.dll"));
        if !target_names.insert(format!("{name}.dll")) {
            cu::bail!("duplicate target name: {name}");
        }
    }

    for name in &config.windows.gnu.extra_links {
        link_files.push(format!("llvm-mingw\\bin\\{name}"));
        if !target_names.insert(name.to_owned()) {
            cu::bail!("duplicate target name: {name}");
        }
    }
    for name in &config.windows.gnu.extra_shims {
        shim_files_gnu.push(format!("llvm-mingw\\bin\\{name}"));
        if !target_names.insert(name.to_owned()) {
            cu::bail!("duplicate target name: {name}");
        }
    }
    for name in &config.windows.gnu.extra_bash_wrapped {
        bash_wrap_files_gnu.push(format!("llvm-mingw\\bin\\{name}"));
        if !target_names.insert(name.to_owned()) {
            cu::bail!("duplicate target name: {name}");
        }
    }
    for name in &config.windows.gnu.extra_python_wrapped {
        python_wrap_files_gnu.push(format!("llvm-mingw\\bin\\{name}"));
        if !target_names.insert(name.to_owned()) {
            cu::bail!("duplicate target name: {name}");
        }
    }
    for name in &config.windows.msvc.extra_links {
        link_files.push(format!("llvm\\bin\\{name}"));
        if !target_names.insert(name.to_owned()) {
            cu::bail!("duplicate target name: {name}");
        }
    }
    for name in &config.windows.msvc.extra_shims {
        shim_files_msvc.push(format!("llvm\\bin\\{name}"));
        if !target_names.insert(name.to_owned()) {
            cu::bail!("duplicate target name: {name}");
        }
    }
    for name in &config.windows.msvc.extra_bash_wrapped {
        bash_wrap_files_msvc.push(format!("llvm\\bin\\{name}"));
        if !target_names.insert(name.to_owned()) {
            cu::bail!("duplicate target name: {name}");
        }
    }
    for name in &config.windows.msvc.extra_python_wrapped {
        python_wrap_files_msvc.push(format!("llvm\\bin\\{name}"));
        if !target_names.insert(name.to_owned()) {
            cu::bail!("duplicate target name: {name}");
        }
    }
    cu::info!("cctools will create {} binaries", target_names.len());
    cu::debug!("cctools binaries: {target_names:?}");

    fn unwrap_file(s: &str) -> cu::Result<&str> {
        if let Some(s) = s.strip_prefix("llvm\\bin\\") {
            return Ok(s);
        }
        if let Some(s) = s.strip_prefix("llvm-mingw\\bin\\") {
            return Ok(s);
        }
        cu::bail!("unexpected file not in llvm or llvm-mingw: {s}");
    }

    for file in link_files {
        if !source_files.contains(&file) {
            cu::bail!("link file not found in installation: {file}");
        }
        let to = install_dir.join(&file).into_utf8()?;
        let from = hmgr::paths::binary(unwrap_file(&file)?).into_utf8()?;
        ctx.add_item(Item::link_bin(from, to))?;
    }
    for file in shim_files_gnu {
        if !source_files.contains(&file) {
            cu::bail!("link file not found in installation: {file}");
        }
        let to = install_dir.join(&file).into_utf8()?;
        let file = unwrap_file(&file)?;
        cu::ensure!(file.ends_with(".exe"), "{file:?}")?;
        ctx.add_item(Item::shim_bin(
            file,
            ShimCommand::target(to).paths([llvm_mingw_dir_str]),
        ))?;
    }
    for file in shim_files_msvc {
        if !source_files.contains(&file) {
            cu::bail!("link file not found in installation: {file}");
        }
        let to = install_dir.join(&file).into_utf8()?;
        let file = unwrap_file(&file)?;
        cu::ensure!(file.ends_with(".exe"), "{file:?}")?;
        ctx.add_item(Item::shim_bin(
            file,
            ShimCommand::target(to).paths([llvm_msvc_dir_str]),
        ))?;
    }
    for (target, source) in shim_rename_files_gnu {
        let to = install_dir.join(&source).into_utf8()?;
        cu::ensure!(target.ends_with(".exe"), "{target:?}")?;
        ctx.add_item(Item::shim_bin(
            target,
            ShimCommand::target(to).paths([llvm_mingw_dir_str]),
        ))?;
    }
    for (target, source) in shim_rename_files_msvc {
        let to = install_dir.join(&source).into_utf8()?;
        cu::ensure!(target.ends_with(".exe"), "{target:?}")?;
        ctx.add_item(Item::shim_bin(
            target,
            ShimCommand::target(to).paths([llvm_msvc_dir_str]),
        ))?;
    }
    for file in bash_wrap_files_gnu {
        let to = install_dir.join(&file).into_utf8()?;
        let file = unwrap_file(&file)?;
        cu::ensure!(!file.ends_with(".exe"), "{file:?}")?;
        ctx.add_item(Item::shim_bin(
            bin_name!(file),
            ShimCommand::target(to).paths([llvm_mingw_dir_str]).bash(),
        ))?;
    }
    for file in bash_wrap_files_msvc {
        let to = install_dir.join(&file).into_utf8()?;
        let file = unwrap_file(&file)?;
        cu::ensure!(!file.ends_with(".exe"), "{file:?}")?;
        ctx.add_item(Item::shim_bin(
            bin_name!(file),
            ShimCommand::target(to).paths([llvm_msvc_dir_str]).bash(),
        ))?;
    }
    for file in python_wrap_files_gnu {
        let to = install_dir.join(&file).into_utf8()?;
        let file = unwrap_file(&file)?;
        cu::ensure!(!file.ends_with(".exe"), "{file:?}")?;
        ctx.add_item(Item::shim_bin(
            bin_name!(file),
            ShimCommand::target("python")
                .args([to])
                .paths([llvm_mingw_dir_str]),
        ))?;
    }
    for file in python_wrap_files_msvc {
        let to = install_dir.join(&file).into_utf8()?;
        let file = unwrap_file(&file)?;
        cu::ensure!(!file.ends_with(".exe"), "{file:?}")?;
        ctx.add_item(Item::shim_bin(
            bin_name!(file),
            ShimCommand::target("python")
                .args([to])
                .paths([llvm_msvc_dir_str]),
        ))?;
    }

    bar.done();

    Ok(())
}

fn configure_cmake(ctx: &Context) -> cu::Result<()> {
    let install_dir = ctx.install_dir();
    let cmake_dir = install_dir.join("cmake");
    let cmake_bin_dir = cmake_dir.join("bin");
    for file_name in ["cmake", "cmcldeps", "cpack", "ctest", "cmake-gui"] {
        let file_name = bin_name!(file_name);
        let from = hmgr::paths::binary(&file_name).into_utf8()?;
        let to = cmake_bin_dir.join(file_name).into_utf8()?;
        ctx.add_item(Item::link_bin(from, to))?;
    }
    Ok(())
}

fn configure_ninja(ctx: &Context) -> cu::Result<()> {
    let install_dir = ctx.install_dir();
    let ninja_dir = install_dir.join("ninja");
    let from = hmgr::paths::binary("ninja.exe").into_utf8()?;
    let to = ninja_dir.join("ninja.exe").into_utf8()?;
    ctx.add_item(Item::link_bin(from, to))?;
    Ok(())
}

pub fn config_location(ctx: &Context) -> cu::Result<Option<PathBuf>> {
    Ok(Some(ctx.config_file()))
}

static CONFIG: ConfigDef<Config> = ConfigDef::new(include_str!("config.toml"), &[]);
test_config!(CONFIG);

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Config {
    pub windows: ConfigWindows,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct ConfigWindows {
    pub gnu: ExtraFileOptions,
    pub msvc: ExtraFileOptions,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct ExtraFileOptions {
    #[serde(default)]
    pub extra_links: Vec<String>,
    #[serde(default)]
    pub extra_shims: Vec<String>,
    #[serde(default)]
    pub extra_bash_wrapped: Vec<String>,
    #[serde(default)]
    pub extra_python_wrapped: Vec<String>,
}
