use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static HOME_PATH: OnceLock<PathBuf> = OnceLock::new();

macro_rules! home {
    ($(,)?) => {};
    ($(,)? $f:ident: $path:literal $($rest:tt)*) => {
#[inline(always)]
pub fn $f() -> PathBuf {
    home().join($path)
}
        home!{$($rest)*}
    };
    ($(,)? $f:ident: ($path:expr) $($rest:tt)*) => {
#[inline(always)]
pub fn $f() -> PathBuf {
    home().join($path)
}
        home!{$($rest)*}
    };
    ($(,)? $f:ident: $root:ident / $path:ident $($rest:tt)*) => {
#[inline(always)]
pub fn $f($path: impl AsRef<Path>) -> PathBuf {
    let mut x = $root();x.push($path);x
}
        home!{$($rest)*}
    };
    ($(,)? $f:ident: $root:ident / $path:literal $($rest:tt)*) => {
#[inline(always)]
pub fn $f() -> PathBuf {
    let mut x = $root();x.push($path);x
}
        home!{$($rest)*}
    };
    ($(,)? $f:ident: $root:ident / ($path:expr) $($rest:tt)*) => {
#[inline(always)]
pub fn $f() -> PathBuf {
    let mut x = $root();x.push($path);x
}
        home!{$($rest)*}
    };
}

/// Initialize the SHAFT_HOME directory path.
///
/// Will fail silently and print a warning if it's already set
pub fn init_home_path(path: PathBuf) {
    cu::debug!("initializing home path: {}", path.display());
    if HOME_PATH.set(path).is_err() {
        cu::warn!(
            "SHAFT_HOME is already initialized at '{}'",
            HOME_PATH.get().unwrap().display()
        )
    }
}

pub(crate) fn home() -> &'static Path {
    HOME_PATH
        .get()
        .expect("home not initialized; please debug with -vv")
}

#[rustfmt::skip]
home! {
    bin_root:              "bin",
    binary:                   bin_root / file,
    config_root:           "config",
    config_toml:              config_root / "core.toml",
    install_root:          "install",
    install_dir:              install_root / package,
    install_old_root:      "install-old",
    install_old_dir:          install_old_root / package,
    items_root:            "items",
    items_config_json:        items_root / "config.json",
    init_ps1:                 items_root / "init.ps1",
    init_bash:                items_root / "init.bash",
    init_zsh:                 items_root / "init.zsh",
    shim_binary:              items_root / (crate::bin_name!("shaftim")),
    shim_binary_old:          items_root / (crate::bin_name!("shaftim.old")),
    shim_config_json:         items_root / "shim_config.json",
    download_root:         "download",
    temp_root:             "temp",
    temp_dir:                 temp_root / path,
    tools_root:            "tools",
    tools_version:            tools_root / "version",
    dot_interrupted:       ".interrupted",
    dot_lock:              ".lock",
    environment_json:      "environment.json",
    install_cache_json:    "install_cache.json",
    previous_command_json: "previous_command.json",
    shaft_binary:          (crate::bin_name!("shaft")),
    shaft_binary_old:      (crate::bin_name!("shaft.old")),
    version_cache_json:    "version_cache.json",
}

/// HOME/config/pkg.toml
#[inline(always)]
pub fn config_file(package: &str) -> PathBuf {
    let mut p = config_root();
    p.push(format!("{package}.toml"));
    p
}

/// HOME/download/<identifier_stem>-<url_hash>.<ext>
#[inline(always)]
pub fn download(identifier: impl AsRef<Path>, url: impl AsRef<str>) -> PathBuf {
    download_file_impl(identifier.as_ref(), url.as_ref())
}
fn download_file_impl(identifier: &Path, url: &str) -> PathBuf {
    let hash = fxhash::hash64(url);
    let mut path_part = OsString::new();
    if let Some(stem) = identifier.file_stem() {
        path_part.push(stem);
        path_part.push("-");
    }
    path_part.push(format!("{hash:016x}"));
    if let Some(ext) = identifier.extension() {
        path_part.push(".");
        path_part.push(ext);
    }
    let mut path = download_root();
    path.push(path_part);
    path
}

#[inline(always)]
pub fn clean_temp_dir(package: impl AsRef<Path>) {
    clean_temp_dir_impl(package.as_ref())
}
fn clean_temp_dir_impl(package: &Path) {
    if let Err(e) = cu::fs::rec_remove(temp_dir(package)) {
        cu::warn!("failed to remove temp dir: {e:?}");
    }
}
