use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::Duration;

use cu::pre::*;

/// Check if current process has elevated privilege
#[cfg(windows)]
pub fn is_sudo() -> bool {
    use winapi::shared::minwindef::{BOOL, DWORD, FALSE, LPVOID};
    use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
    use winapi::um::securitybaseapi::GetTokenInformation;
    use winapi::um::winnt::{
        HANDLE, TOKEN_ELEVATION, TOKEN_ELEVATION_TYPE, TOKEN_QUERY, TokenElevation,
    };
    // https://github.com/yandexx/is_elevated/blob/master/src/lib.rs
    // based on https://stackoverflow.com/a/8196291
    let mut token_handle = HANDLE::default();
    let process = unsafe { GetCurrentProcess() };
    let success: BOOL = unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut token_handle) };
    if success == FALSE {
        return false;
    }

    let mut token_elevation = TOKEN_ELEVATION::default();
    let mut size: DWORD = 0;
    let success = unsafe {
        GetTokenInformation(
            token_handle,
            TokenElevation,
            &mut token_elevation as *mut TOKEN_ELEVATION as LPVOID,
            std::mem::size_of::<TOKEN_ELEVATION_TYPE>() as u32,
            &mut size,
        )
    };
    if success == FALSE {
        return false;
    }
    token_elevation.TokenIsElevated != 0
}

/// Check if current process has elevated privilege
#[cfg(not(windows))]
pub fn is_sudo() -> bool {
    unsafe { libc::geteuid() == 0 }
}

/// Find the binary in path and make a sudo command for it
///
/// On non-Windows, it will check and prompt for sudo password if needed.
/// On Windows, currently a UAC prompt still shows every time a command is launched with sudo.exe
pub fn sudo(binary: &str, reason: &str) -> cu::Result<cu::Command<(), (), ()>> {
    let lower = binary.to_lowercase();
    // this is not a security guarantee as there can be other extensions runnable on Windows
    if lower == "sudo" || lower == "sudo.exe" {
        cu::bail!("cannot run sudo with sudo");
    }
    let path = cu::which(binary)?;
    sudo_path_name(&path, binary, reason)
}

/// Make a sudo command for the binary at path
///
/// On non-Windows, it will check and prompt for sudo password if needed.
/// On Windows, currently a UAC prompt still shows every time a command is launched with sudo.exe
pub fn sudo_path(path: &Path, reason: &str) -> cu::Result<cu::Command<(), (), ()>> {
    let name = path
        .file_name()
        .and_then(|x| x.to_str())
        .unwrap_or_default();
    sudo_path_name(&path, name, reason)
}

fn sudo_path_name(path: &Path, name: &str, reason: &str) -> cu::Result<cu::Command<(), (), ()>> {
    cu::warn!(
        "[sudo] will spawn this executable: {}\n- reason: {}",
        path.display(),
        reason
    );
    #[cfg(not(windows))]
    {
        validate_credential()?;
    }
    #[cfg(windows)]
    {
        // when UAC shows, the terminal will not be visible, so I will have no idea
        // what sudo.exe is trying to do, so, spawn a prompt here to let me ack that
        // a UAC will show
        //
        // this also allows --non-interactive to fail here
        // note we use prompt! instead of yesno!, because we don't want -y to bypass
        // this automatically
        let mut answer = cu::prompt!(
            "[sudo] enter 'ok' to allow. A User Access Control (UAC) will show that runs 'sudo.exe'"
        )?;
        while answer != "ok" {
            cu::error!("please enter 'ok'");
            cu::warn!(
                "[sudo] will spawn this executable: {}\n- reason: {}",
                path.display(),
                reason
            );
            answer = cu::prompt!(
                "[sudo] enter 'ok' to allow. A User Access Control (UAC) will show that runs 'sudo.exe'"
            )?;
        }
    }
    let mut command = which_sudo()?.command();
    if !name.is_empty() {
        command = command.name(name);
    }
    Ok(command.arg(path))
}

#[allow(unused)]
fn validate_credential() -> cu::Result<()> {
    let sudo_path = which_sudo()?;
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

static SUDO_PATH: LazyLock<Result<PathBuf, String>> =
    LazyLock::new(|| init_sudo_path().map_err(|e| format!("{e:?}")));
#[cfg(windows)]
fn init_sudo_path() -> cu::Result<PathBuf> {
    // only load sudo.exe at the expected location
    let mut sudo_path = PathBuf::from(format!("{}\\", cu::env_var("SystemDrive")?));
    sudo_path.extend(["Windows", "System32", "sudo.exe"]);
    if !sudo_path.is_file() {
        cu::bail!("cannot find sudo.exe. Please ensure Sudo for Windows is enabled.");
    }
    let path2 = cu::which("sudo")?;
    if sudo_path != path2 {
        cu::error!(
            "sudo.exe is not at the expected location. Either you are running an unsupported version of Windows, or your PATH is corrupted (possibly by a malicous program)"
        );
        cu::bail!("refusing to run suspicous path: {}", path2.display());
    }
    // we could verify the signature here..
    // but the easiest way is to use an external program like powershell
    // but how do we verify that program is legit?
    // we probably have to trust protected directories on Windows :<
    Ok(sudo_path)
}

#[cfg(not(windows))]
fn init_sudo_path() -> cu::Result<PathBuf> {
    let sudo_path = PathBuf::from("/usr/bin/sudo");
    if !sudo_path.is_file() {
        cu::bail!("cannot find sudo.");
    }
    let path2 = cu::which("sudo")?;
    if sudo_path != path2 {
        cu::error!(
            "sudois not at the expected location. Your PATH might be corrupted (possibly by a malicous program)"
        );
        cu::bail!("refusing to run suspicous path: {}", path2.display());
    }
    Ok(sudo_path)
}

/// Get the path of sudo
pub fn which_sudo() -> cu::Result<PathBuf> {
    match &*SUDO_PATH {
        Ok(x) => Ok(x.clone()),
        Err(e) => cu::bail!("sudo verification failed: {e}"),
    }
}
