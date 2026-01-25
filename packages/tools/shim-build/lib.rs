use std::ffi::OsStr;
use std::path::Path;

#[inline(always)]
pub fn exe_name(s: &OsStr) -> &[u8] {
    match Path::new(s).file_name() {
        Some(name) => name.as_encoded_bytes(),
        None => &[]
    }
}

pub use imp::exec_replace;

// Reference
// https://github.com/rust-lang/cargo/blob/master/crates/cargo-util/src/process_builder.rs
#[cfg(unix)]
mod imp {
    use std::process::{ExitCode, Command};
    use std::os::unix::process::CommandExt;
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
    use std::process::{ExitCode, Command};

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
        let success = unsafe {
            SetConsoleCtrlHandler(Some(ctrlc_handler), TRUE)
        };
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
