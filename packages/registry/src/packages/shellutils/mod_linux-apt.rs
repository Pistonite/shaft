
use crate::pre::*;

#[rustfmt::skip]
register_binaries!(
    "perl", "gpg", "curl", "wget",
    "fzf", "jq", "task", "x",
    "bat", "dust", "fd", "rg", "websocat", "zoxide", "c", "ci",
    "viopen", "vibash", "vihosts", "n", "lfmt"
);
binary_dependencies!(CargoBinstall);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_in_path!("perl");
    check_verified!(perl::version_check()?);
    cu::check!(
        cu::which("gpg"),
        "gnupg is a dependency of Arch Linux and is not found"
    )?;

    let v = check_in_path!("curl");
    check_outdated!(&v, metadata[curl]::VERSION);
    let v = check_pacman!("wget");
    check_outdated!(&v, metadata[wget]::VERSION);

    check_verified!(fzf::verify()?);
    check_verified!(jq::verify()?);
    check_verified!(task::verify()?);
    check_verified!(cargoones::verify()?);
    check_verified!(shutil::verify()?);

    check_config_version_cache!(common::ALIAS_VERSION);
    Ok(Verified::UpToDate)
}
