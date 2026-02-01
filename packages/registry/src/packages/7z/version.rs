use crate::pre::*;

pub fn check(expected_version: &str) -> cu::Result<Verified> {
    for line in command_output!("7z").lines() {
        let Some(rest) = line.strip_prefix("7-Zip ") else {
            continue;
        };
        let mut parts = rest.split(' ');
        let Some(version) = parts.next() else {
            break;
        };
        if Version(version).lt(expected_version) {
            return Ok(Verified::NotUpToDate);
        }
        return Ok(Verified::UpToDate);
    }
    cu::warn!("failed to parse current version for '7z'");
    Ok(Verified::NotUpToDate)
}
