/// Verify a binary is in PATH. Takes the name of the executable.
///
/// Returns the path of the binary
macro_rules! check_in_path {
    ($l:literal) => {
        match cu::which($l) {
            Ok(p) => p,
            Err(_) => {
                cu::error!("verify: not found in PATH: '{}'", $l);
                return Ok(Verified::NotInstalled);
            }
        }
    };
}
pub(crate) use check_in_path;

/// Verify a binary is in PATH and is in the shaft's binary directory
/// Takes the name of the executable.
///
/// Optionally, takes the name of the system package that can be used as alternative
macro_rules! check_in_shaft {
    ($bin:literal) => {{
        match cu::which($bin) {
            Err(e) => {
                cu::error!("verify: not found in PATH: '{}'", $bin);
                cu::debug!("check_in_shaft failed: {e:?}");
                return Ok(Verified::NotInstalled);
            }
            Ok(path) => {
                if path != hmgr::paths::binary(bin_name!($bin)) {
                    cu::bail!(
                        "found existing '{}' installed outside of shaft, please uninstall it first (at '{}'), or ensure the shaft bin has higher priority in PATH",
                        $bin,
                        path.display()
                    );
                }
                path
            }
        }
    }};
    ($bin:literal || $system:literal) => {{
        match cu::which($bin) {
            Err(e) => {
                cu::error!("verify: not found in PATH: '{}'", $bin);
                cu::debug!("check_in_shaft failed: {e:?}");
                return Ok(Verified::NotInstalled);
            }
            Ok(path) => {
                if path != hmgr::paths::binary(bin_name!($bin)) {
                    cu::bail!(
                        "found existing '{}' installed outside of shaft, please uninstall it first (at '{}'), or ensure the shaft bin has higher priority in PATH; alternatively, use the {} package",
                        $bin,
                        path.display(),
                        $system
                    );
                }
                path
            }
        }
    }};
}
pub(crate) use check_in_shaft;

/// Check cargo install metadata for the crate or binary
///
/// ```rust,ignore
/// check_cargo!("binary"); // crate name is the same as binary
/// check_cargo!("binary" in crate "crate");
/// ```
macro_rules! check_cargo {
    ($bin:literal) => {{ check_cargo!($bin in crate $bin) }};
    ($bin:literal in crate $l:literal) => {{
        if cu::which($bin).is_err() {
        cu::error!("verify: not found in PATH: '{}'", $bin);
            cu::debug!("check_cargo failed: binary not found: {} (crate {})", $bin, $l);
            return Ok(Verified::NotInstalled);
        }
        match epkg::cargo::installed_info($l)? {
            None => {
                cu::bail!(
                    "current '{}' is not installed with cargo; please uninstall it first, so we can install the '{}' crate",
                    $bin, $l
                )
            }
            Some(info) => info,
        }
    }};
}
pub(crate) use check_cargo;

/// Check pacman install metadata for a pacman package
#[cfg(target_os = "linux")]
macro_rules! check_pacman {
    ($l:literal) => {
        match epkg::pacman::installed_version($l)? {
            None => {
                cu::error!("verify: pacman package not installed: '{}'", $l);
                return Ok(Verified::NotInstalled);
            }
            Some(x) => x,
        }
    };
}
#[cfg(target_os = "linux")]
pub(crate) use check_pacman;

/// Check actual version is at least as new as expected version
macro_rules! check_outdated {
    ($actual:expr, metadata [ $($package:ident)::* ]:: $($expected:tt)*) => {{
        let a = $actual;
        let e = metadata::$($package)::*::$($expected)*;
        if Version(a).lt(e) {
            cu::error!("verify: {} {} is outdated, new version: {}", stringify!($($package).*), a, e);
            return Ok(Verified::NotUpToDate);
        }
    }};
    ($actual:expr, $expected:expr) => {{
        let a = $actual;
        let e = $expected;
        if Version(a).lt(e) {
            cu::error!("verify: {} {} is outdated, new version: {}", stringify!($expected), a, e);
            return Ok(Verified::NotUpToDate);
        }
    }};
}
pub(crate) use check_outdated;

/// Check the status of a sub `Verified`
macro_rules! check_verified {
    ($sub:expr) => {{
        let v = $sub;
        if v != Verified::UpToDate {
            cu::debug!("check_verified: for '{}': {:?}", stringify!($sub), v);
            return Ok(v);
        }
    }};
}
pub(crate) use check_verified;

/// Check status if a version cache
macro_rules! check_version_cache {
    ($cache:expr) => {{
        let cache = $cache;
        match cache.is_uptodate()? {
            None => {
                cu::error!("verify: new config: {} = {}", cache.id(), cache.version());
                return Ok(Verified::NotInstalled);
            }
            Some(false) => {
                cu::error!(
                    "verify: config {} is bumped: {}",
                    cache.id(),
                    cache.version()
                );
                return Ok(Verified::NeedsConfig);
            }
            _ => {}
        }
    }};
}
pub(crate) use check_version_cache;

/// Create a scope where any verification failure (NotUpToDate or NotInstalled) will be turned into NeedsConfig
#[allow(unused)]
macro_rules! verify_config {
    ($($s:tt)*) => {{
        match (|| -> cu::Result<Verified> { $($s)* })() {
            Ok(Verified::UpToDate) => Ok(Verified::UpToDate),
            Ok(_) => Ok(Verified::NeedsConfig),
            Err(x) => Err(x)
        }
    }}
}
#[allow(unused)]
pub(crate) use verify_config;
