use crate::pre::*;

pub fn verify(min_version_req: &str, require_vfs: bool) -> cu::Result<Verified> {
    let stdout = command_output!("git", ["--version"]);
    for line in stdout.lines() {
        let Some(version) = line.strip_prefix("git version ") else {
            continue;
        };
        // source token is used in the macro for info messages
        let git = min_version_req;
        check_outdated!(version, git);
        if require_vfs && !version.contains("vfs") {
            cu::bail!(
                "current 'git' is not the vfs version (microsoft.git); please uninstall it or use the 'system-git' package"
            );
        }
        return Ok(Verified::UpToDate);
    }
    cu::bail!("failed to get git version from output: {stdout}");
}
