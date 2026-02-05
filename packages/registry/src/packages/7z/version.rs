use crate::pre::*;

pub fn check() -> cu::Result<Verified> {
    for line in command_output!("7z").lines() {
        let Some(rest) = line.strip_prefix("7-Zip ") else {
            continue;
        };
        let mut parts = rest.split(' ');
        let Some(version) = parts.next() else {
            break;
        };
        check_outdated!(version, metadata[_7z]::VERSION);
        return Ok(Verified::UpToDate);
    }
    cu::warn!("failed to parse current version for '7z'");
    Ok(Verified::NotUpToDate)
}
