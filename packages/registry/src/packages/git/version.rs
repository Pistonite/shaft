use crate::pre::*;

pub fn verify() -> cu::Result<Verified> {
    let stdout = command_output!("git", ["--version"]);
    for line in stdout.lines() {
        let Some(version) = line.strip_prefix("git version ") else {
            continue;
        };
        check_outdated!(version, metadata[git]::VERSION);
        return Ok(Verified::UpToDate);
    }
    cu::bail!("failed to get git version from output: {stdout}");
}
