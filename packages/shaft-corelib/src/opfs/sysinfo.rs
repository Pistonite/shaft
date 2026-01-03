use std::time::Duration;

use cu::pre::*;
use sysinfo::{Pid, System};

use crate::internal;

#[derive(Deref, DerefMut)]
pub(crate) struct State(System);
impl State {
    pub fn new() -> cu::Result<Self> {
        Ok(Self(System::new()))
    }
}

internal::main_thread_singleton! {
    let system = State::new();
}

/// Ensure no process with the given exe file name is running. Wait for it to terminate
/// up to some time if it is running.
///
/// Note that the process name passed in needs to be platform-specific,
/// for example `git` on Linux and `git.exe` on Windows
#[inline(always)]
pub fn ensure_terminated(exe_name: impl AsRef<str>) -> cu::Result<()> {
    ensure_terminated_impl(exe_name.as_ref())
}
fn ensure_terminated_impl(exe_name: &str) -> cu::Result<()> {
    let mut s = system::instance()?;
    let Some(pid) = get_process_pid(&mut s, exe_name) else {
        return Ok(());
    };
    cu::warn!("'{exe_name}' (pid={pid}) is running, waiting for it to be terminated...");
    for _ in 0..5 {
        std::thread::sleep(Duration::from_secs(1));
        let Some(pid) = get_process_pid(&mut s, exe_name) else {
            return Ok(());
        };
        cu::warn!("'{exe_name}' (pid={pid}) is still running...");
    }
    cu::bail!("'{exe_name}' did not terminate - please retry after stopping the process manually");
}

fn get_process_pid(s: &mut System, exe_name: &str) -> Option<Pid> {
    s.refresh_processes(sysinfo::ProcessesToUpdate::All, true /* remove_dead */);
    for (pid, process) in s.processes() {
        let Some(exe) = process.exe() else {
            continue;
        };
        let Some(filename) = exe.file_name() else {
            continue;
        };
        if filename == exe_name {
            return Some(*pid);
        }
    }
    None
}
