use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static HOME_PATH: OnceLock<PathBuf> = OnceLock::new();

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

fn home() -> &'static Path {
    HOME_PATH
        .get()
        .expect("home not initialized; please debug with -vv")
}

/// HOME/install_cache.json
#[inline(always)]
pub fn install_cache_json() -> PathBuf {
    home().join("install_cache_json")
}

/// HOME/shaft or HOME/shaft.exe
#[inline(always)]
pub fn shaft_binary() -> PathBuf {
    home().join(crate::bin_name!("shaft"))
}

/// HOME/shaft.old or HOME/shaft.old.exe
#[inline(always)]
pub fn shaft_binary_old() -> PathBuf {
    home().join(crate::bin_name!("shaft.old"))
}

/// HOME/environment.json
#[inline(always)]
pub fn environment_json() -> PathBuf {
    home().join("environment.json")
}

/// HOME/previous_command.json
#[inline(always)]
pub fn previous_command_json() -> PathBuf {
    home().join("previous_command.json")
}

/// HOME/config.toml
#[inline(always)]
pub fn config_toml() -> PathBuf {
    home().join("config.toml")
}

/// HOME/.interruped
#[inline(always)]
pub fn dot_interrupted() -> PathBuf {
    home().join(".interrupted")
}

/// HOME/.lock
#[inline(always)]
pub fn dot_lock() -> PathBuf {
    home().join(".lock")
}

/// HOME/init/
#[inline(always)]
pub fn init_root() -> PathBuf {
    home().join("init")
}

/// HOME/bin/
#[inline(always)]
pub fn bin_root() -> PathBuf {
    home().join("bin")
}

/// HOME/temp/
#[inline(always)]
pub fn temp_root() -> PathBuf {
    home().join("temp")
}

/// HOME/temp/<package>
#[inline(always)]
pub fn temp_dir(package: impl AsRef<Path>) -> PathBuf {
    let mut x = temp_root();
    x.push(package);
    x
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
