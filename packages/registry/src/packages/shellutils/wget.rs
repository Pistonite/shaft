use crate::pre::*;

pub fn version_check() -> cu::Result<Verified> {
    if cu::which("wget").is_err() {
        return Ok(Verified::NotInstalled);
    }
    let stdout = command_output!("wget", ["--version"]);
    // "GNU Wget 1.25.0.230 ..."
    for line in stdout.lines() {
        let Some(rest) = line.strip_prefix("GNU Wget ") else {
            continue;
        };
        let Some(version) = rest.split_whitespace().next() else {
            continue;
        };
        return Ok(Verified::is_uptodate(
            !(Version(version).lt(metadata::wget::VERSION)),
        ));
    }
    cu::bail!("failed to get wget version from output: {stdout}");
}
