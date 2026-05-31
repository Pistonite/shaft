use crate::pre::*;

pub fn verify(require_vfs: bool) -> cu::Result<Verified> {
    let stdout = command_output!("git", ["--version"]);
    for line in stdout.lines() {
        let Some(version) = line.strip_prefix("git version ") else {
            continue;
        };
        check_outdated!(version, metadata[git]::VERSION);
        if require_vfs && !version.contains("vfs") {
            cu::bail!(
                "current 'git' is not the vfs version (microsoft.git); please uninstall it or use the 'system-git' package"
            );
        }
        return Ok(Verified::UpToDate);
    }
    cu::bail!("failed to get git version from output: {stdout}");
}
