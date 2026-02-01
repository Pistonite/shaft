use crate::pre::*;

pub fn verify(min_version: &str) -> cu::Result<Verified> {
    if cu::which("git").is_err() {
        return Ok(Verified::NotInstalled);
    }
    let stdout = command_output!("git", ["--version"]);
    for line in stdout.lines() {
        let Some(version) = line.strip_prefix("git version ") else {
            continue;
        };
        return Ok(Verified::is_uptodate(!(Version(version).lt(min_version))));
    }
    cu::bail!("failed to get git version from output: {stdout}");
}
