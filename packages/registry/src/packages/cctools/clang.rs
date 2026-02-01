use crate::pre::*;

pub fn verify(ctx: &Context) -> cu::Result<Verified> {
    if cfg!(windows) {
        check_bin_in_path_and_shaft!("gcc", "system-cctools");
        check_bin_in_path_and_shaft!("gcc++", "system-cctools");
        check_bin_in_path_and_shaft!("clang", "system-cctools");
        check_bin_in_path_and_shaft!("clang++", "system-cctools");
        check_bin_in_path_and_shaft!("clang-format", "system-cctools");
        check_bin_in_path_and_shaft!("clang-tidy", "system-cctools");
    } else {
        check_bin_in_path!("clang");
        check_bin_in_path!("clang-format");
        check_bin_in_path!("clang-tidy");
    }
    let output = command_output!("clang", ["--version"]);
    let info = ClangInfo::parse(&output);
    if info.version.is_empty() {
        cu::warn!("failed to determine clang version");
        return Ok(Verified::NotUpToDate);
    }
    if cfg!(windows) {
        let expected_install_dir = ctx.install_dir().join("bin");
        let actual_install_dir = Path::new(&info.install_dir).normalize()?;
        if expected_install_dir != actual_install_dir {
            cu::bail!(
                "detected existing clang installation, please uninstall it or use system-cctools"
            );
        }
    }
    if Version(&info.version).lt(metadata::clang::LLVM_VERSION) {
        return Ok(Verified::NotUpToDate);
    }
    Ok(Verified::UpToDate)
}

struct ClangInfo {
    version: String,
    install_dir: String,
}

impl ClangInfo {
    fn parse(s: &str) -> Self {
        let mut version = None;
        let mut install_dir = None;

        for line in s.lines() {
            if let Some(rest) = line.strip_prefix("clang version ") {
                // Version might have additional info after it (like git URL on Windows)
                // Just take the first word (the version number)
                version = rest.split_whitespace().next().map(|s| s.to_string());
            } else if let Some(rest) = line.strip_prefix("InstalledDir: ") {
                install_dir = Some(rest.to_string());
            }
        }

        Self {
            version: version.unwrap_or_default(),
            install_dir: install_dir.unwrap_or_default(),
        }
    }
}
