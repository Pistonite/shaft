use cu::pre::*;
use op::Version;

use crate::pre::*;

pub fn check(expected_version: &str) -> cu::Result<Verified> {
    let (child, stdout) = cu::which("7z")?.command()
    .stdout(cu::pio::string())
        .stdie_null()
        .spawn()?;
    child.wait_nz()?;
    let stdout = stdout.join()??;
    for line in stdout.lines() {
        let Some(rest) = line.strip_prefix("7-Zip ") else {
            continue;
        };
        let mut parts = rest.split(' ');
        let Some(version) = parts.next() else {
            break;
        };
        if Version(version) >= expected_version {
            return Ok(Verified::UpToDate);
        }
    }
    cu::warn!("failed to parse current version for '7z'");
    Ok(Verified::NotUpToDate)
}
