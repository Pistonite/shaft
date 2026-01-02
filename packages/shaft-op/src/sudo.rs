use std::path::Path;
use std::time::Duration;

use cu::pre::*;

/// Make a sudo command
///
/// On non-Windows, it will check and prompt for sudo password if needed.
/// On Windows, currently a UAC prompt still shows every time a command is launched with sudo.exe
pub fn command(arg0: &str) -> cu::Result<cu::Command<(), (), ()>> {
    #[cfg(not(windows))]
    {
        validate_credential()?;
    }
    let path = cu::which(arg0)?;
    Ok(cu::which("sudo")?.command().name(arg0).arg(path))
}

#[allow(unused)]
fn validate_credential() -> cu::Result<()> {
    let sudo_path = cu::which("sudo")?;
    // check if user's cached credential is valid
    if let Ok(true) = check_credential(&sudo_path) {
        return Ok(());
    }
    let prompt = match get_user_name().ok() {
        Some(x) => {
            format!("[sudo] password for {x}")
        }
        None => {
            format!("[sudo] password")
        }
    };
    let mut secs = 1;
    loop {
        if let Err(e) = refresh_credential(&sudo_path, &prompt, Duration::from_secs(secs)) {
            cu::error!("{e:?}");
        }
        match check_credential(&sudo_path) {
            Ok(true) => return Ok(()),
            Ok(false) => {
                // ... did not work
            }
            Err(e) => {
                cu::error!("{e:?}");
            }
        }
        cu::error!("sorry, try again");
        secs += 1;
    }

    Ok(())
}

fn check_credential(sudo_path: &Path) -> cu::Result<bool> {
    let status = sudo_path.command().args(["-Nnv"]).all_null().wait()?;
    Ok(status.success())
}

fn refresh_credential(sudo_path: &Path, prompt: &str, timeout: Duration) -> cu::Result<()> {
    let password = cu::prompt_password!("{prompt}")?;
    let mut child = sudo_path
        .command()
        .arg("-vS")
        .stdin(cu::pio::write(password))
        .stdoe_null()
        .spawn()?;
    if child.wait_timeout(timeout)?.is_none() {
        child.kill()?;
    } else {
        child.wait()?;
    }
    Ok(())
}

fn get_user_name() -> cu::Result<String> {
    let (child, stdout) = cu::which("whoami")?
        .command()
        .stdout(cu::pio::string())
        .stdie_null()
        .spawn()?;
    let name = stdout.join()??;
    child.wait_nz()?;
    Ok(name.trim().to_string())
}
