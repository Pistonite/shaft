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
                        "found existing '{}' installed outside of shaft, please uninstall it first (at '{}')",
                        $bin,
                        path.display()
                    );
                }
                path
            }
        }
    }};
}
pub(crate) use check_bin_in_path_and_shaft;

#[cfg(target_os = "linux")]
macro_rules! check_installed_with_pacman {
    ($l:literal) => {
        if !corelib::epkg::pacman::is_installed($l)? {
            cu::bail!(concat!(
                "current '",
                $l,
                "' is not installed with pacman; please uninstall it"
            ))
        }
    };
    ($l:literal, $system:literal) => {
        if !corelib::epkg::pacman::is_installed($l)? {
            cu::bail!(concat!(
                "current '",
                $l,
                "' is not installed with pacman; please uninstall it or use the '",
                $system,
                "' package"
            ))
        }
    };
}
#[cfg(target_os = "linux")]
pub(crate) use check_installed_with_pacman;

macro_rules! check_installed_with_cargo {
    ($l:literal) => {{
        check_bin_in_path!($l);
        match corelib::epkg::cargo::installed_info($l)? {
            None => {
                cu::bail!(concat!(
                    "current '",
                    $l,
                    "' is not installed with cargo; please uninstall it"
                ))
            }
            Some(info) => info,
        }
    }};
}
pub(crate) use check_installed_with_cargo;

#[cfg(windows)]
macro_rules! check_installed_with_git {
    ($l:literal, $path:literal) => {{
        let path = check_bin_in_path!($l);
        if corelib::opfs::find_in_wingit($path) != Ok(path) {
            cu::bail!(concat!(
                "current '",
                $l,
                "' is not installed with Git; please uninstall it"
            ))
        }
        path
    }};
}
#[cfg(windows)]
pub(crate) use check_installed_with_git;
