use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::Command;

/// Get the executable name as bytes
#[inline(always)]
pub fn exe_name(s: &OsStr) -> &[u8] {
    match Path::new(s).file_name() {
        Some(name) => name.as_encoded_bytes(),
        None => &[],
    }
}

/// Prepend to the PATH
#[inline(always)]
pub fn set_path(cmd: &mut Command, paths_to_prepend: &str) {
    match std::env::var_os("PATH") {
        Some(path) => {
            let mut new_path = OsString::from(paths_to_prepend);
            new_path.push(paths_to_prepend);
            if cfg!(windows) {
                new_path.push(";");
            } else {
                new_path.push(":");
            }
            new_path.push(&path);
            cmd.env("PATH", path);
        }
        None => {
            cmd.env("PATH", paths_to_prepend);
        }
    }
}

#[cfg(windows)]
pub fn exec_bash_replace(cfg_args: &[&str], cli_args: std::env::ArgsOs, paths_to_prepend: Option<&str>) -> std::process::ExitCode {
    // the library we use only supports utf8
    let mut cli_args_utf8 = Vec::with_capacity(cli_args.len());
    for a in cli_args {
        let Some(a) = a.to_str() else {
            eprintln!("non utf-8 argument: {}", a.display());
            return std::process::ExitCode::FAILURE;
        };
        cli_args_utf8.push(a.to_string());
    }
    let script = shell_words::join(
        cfg_args
            .iter()
            .copied()
            .chain(cli_args_utf8.iter().map(|x| x.as_str())),
    );
    let mut cmd = Command::new("bash.exe");
    cmd.args(["-c", &script]);
    if let Some(p) = paths_to_prepend {
        set_path(&mut cmd, p);
    }
    exec_replace(cmd)
}

pub use imp::exec_replace;

// Reference
// https://github.com/rust-lang/cargo/blob/master/crates/cargo-util/src/process_builder.rs
#[cfg(unix)]
mod imp {
    use std::os::unix::process::CommandExt;
    use std::process::{Command, ExitCode};
    #[inline(always)]
    pub fn exec_replace(mut command: Command) -> ExitCode {
        // execvp
        let error = command.exec();
        eprintln!("execvp failed: {error}");
        ExitCode::from(255)
    }
}
#[cfg(windows)]
mod imp {
    use std::process::{Command, ExitCode};

    use windows_sys::Win32::Foundation::{FALSE, TRUE};
    use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
    use windows_sys::core::BOOL;

    /// Note from cargo-util:
    ///
    /// On Windows this (execvp) isn't technically possible. Instead we emulate it to the best of our
    /// ability. One aspect we fix here is that we specify a handler for the Ctrl-C handler.
    /// In doing so (and by effectively ignoring it) we should emulate proxying Ctrl-C
    /// handling to the application at hand, which will either terminate or handle it itself.
    /// According to Microsoft's documentation at
    /// <https://docs.microsoft.com/en-us/windows/console/ctrl-c-and-ctrl-break-signals>.
    /// the Ctrl-C signal is sent to all processes attached to a terminal, which should
    /// include our child process. If the child terminates then we'll reap them in Cargo
    /// pretty quickly, and if the child handles the signal then we won't terminate
    /// (and we shouldn't!) until the process itself later exits.
    #[inline(always)]
    pub fn exec_replace(mut command: Command) -> ExitCode {
        let success = unsafe { SetConsoleCtrlHandler(Some(ctrlc_handler), TRUE) };
        if success == FALSE {
            eprintln!("execvp: failed to set ctrl-c handler");
            return ExitCode::from(254);
        }
        // exec normally
        let mut child = match command.spawn() {
            Ok(x) => x,
            Err(_) => {
                eprintln!("execvp failed: spawn failed");
                return ExitCode::from(255);
            }
        };
        let exit_status = match child.wait() {
            Ok(x) => x,
            Err(_) => {
                eprintln!("execvp failed: wait failed");
                return ExitCode::from(253);
            }
        };
        let code = exit_status.code().unwrap_or(255) as u8;
        ExitCode::from(code)
    }

    unsafe extern "system" fn ctrlc_handler(_: u32) -> BOOL {
        // Do nothing; let the child process handle it.
        TRUE
    }
}
