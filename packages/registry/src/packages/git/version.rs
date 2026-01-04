use crate::pre::*;

pub fn verify(min_version: &str) -> cu::Result<Verified> {
    let Ok(git) = cu::which("git") else {
        return Ok(Verified::NotInstalled);
    };
    let (child, stdout) = git
        .command()
        .arg("--version")
        .stdout(cu::pio::string())
        .stdie_null()
        .spawn()?;
    child.wait_nz()?;
    let stdout = stdout.join()??;
    let version = stdout.strip_prefix("git version ").unwrap_or(&stdout);

    if Version(version) >= min_version {
        Ok(Verified::UpToDate)
    }
    else {
        Ok(Verified::NotUpToDate)
    }
}

