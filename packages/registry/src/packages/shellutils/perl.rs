use crate::pre::*;

pub fn version_check() -> cu::Result<Verified> {
    if cu::which("perl").is_err() {
        return Ok(Verified::NotInstalled);
    }
    let stdout = command_output!("perl", ["--version"]);
    // "This is perl 5, version 40, subversion 1 (v5.40.1) ..."
    for line in stdout.lines() {
        let Some(i) = line.find("(v") else {
            continue;
        };
        let rest = &line[i + 2..];
        let Some(j) = rest.find(')') else {
            continue;
        };
        let version = &rest[..j];
        let version = version.strip_suffix("*").unwrap_or(version);
        return Ok(Verified::is_uptodate(
            !(Version(version).lt(metadata::perl::VERSION)),
        ));
    }
    cu::bail!("failed to get perl version from output: {stdout}");
}
