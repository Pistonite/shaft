/// Stub macro for build script to generate binaries provided by a package
macro_rules! register_binaries {
    ($($l:literal),*) => {};
}
pub(crate) use register_binaries;

macro_rules! check_bin_in_path {
    ($l:literal) => {
        if cu::which($l).is_err() {
            return Ok(Verified::NotInstalled);
        }
    };
}
pub(crate) use check_bin_in_path;

macro_rules! check_bin_in_path_and_shaft {
    ($bin:literal) => {{
        match cu::which($bin) {
            Err(_) => return Ok(Verified::NotInstalled),
            Ok(path) => {
                if path != corelib::hmgr::paths::binary(corelib::bin_name!($bin)) {
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
    ($bin:literal, $system:literal) => {{
        match cu::which($bin) {
            Err(_) => return Ok(Verified::NotInstalled),
            Ok(path) => {
                if path != corelib::hmgr::paths::binary(corelib::bin_name!($bin)) {
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
pub(crate) use check_bin_in_path_and_shaft;

#[cfg(target_os = "linux")]
macro_rules! check_installed_pacman_package {
    ($l:literal) => {
        match corelib::epkg::pacman::installed_version($l)? {
            None => {
                return Ok(Verified::NotInstalled);
            }
            Some(x) => x,
        }
    };
}
#[cfg(target_os = "linux")]
pub(crate) use check_installed_pacman_package;

#[cfg(target_os = "linux")]
macro_rules! check_installed_with_pacman {
    ($bin:literal, $l:literal) => {
        check_bin_in_path!($bin);
        match corelib::epkg::pacman::installed_version($l)? {
            None => {
                cu::bail!("current '{}' is not installed with pacman; please uninstall it", $bin)
            }
            Some(x) => x,
        }
    };
    ($bin:literal, $l:literal, $system:literal) => {
        check_bin_in_path!($bin);
        match corelib::epkg::pacman::installed_version($l)? {
            None => {
                cu::bail!("current '{}' is not installed with pacman; please uninstall it or use the '{}' package", $bin, $system)
            }
            Some(x) => x,
        }
    };
}
#[cfg(target_os = "linux")]
pub(crate) use check_installed_with_pacman;

macro_rules! check_installed_with_cargo {
    ($bin:literal) => {{ check_installed_with_cargo!($bin, $bin) }};
    ($bin:literal, $l:literal) => {{
        check_bin_in_path!($bin);
        match corelib::epkg::cargo::installed_info($l)? {
            None => {
                cu::bail!(
                    "current '{}' is not installed with cargo; please uninstall it",
                    $bin
                )
            }
            Some(info) => info,
        }
    }};
}
pub(crate) use check_installed_with_cargo;
