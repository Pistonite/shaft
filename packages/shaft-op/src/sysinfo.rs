use std::{
    ffi::OsStr,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    time::Duration,
};

use cu::pre::*;
use sysinfo::System;

crate::main_thread! {
    fn system() -> cu::Result<System> {
        Ok(System::new())
    }
}

/// Ensure no process with the given name is running. Wait for it to terminate
/// up to some time if it is running.
///
/// Note that the process name passed in needs to be platform-specific,
/// for example `git` on Linux and `git.exe` on Windows
pub fn ensure_terminated(process_name: &str) -> cu::Result<()> {
    let mut s = system::instance()?;
    s.refresh_processes(sysinfo::ProcessesToUpdate::All, true /* remove_dead */);
    if s.processes_by_exact_name(process_name.as_ref())
        .next()
        .is_none()
    {
        return Ok(());
    }
    for _ in 0..5 {
        cu::warn!("process '{process_name}' is running, waiting for it to be terminated...");
        std::thread::sleep(Duration::from_secs(1));
        s.refresh_processes(sysinfo::ProcessesToUpdate::All, true /* remove_dead */);
        if s.processes_by_exact_name(process_name.as_ref())
            .next()
            .is_none()
        {
            return Ok(());
        }
    }
    cu::bail!(
        "process '{process_name}' did not terminate - please retry after stopping the process manually"
    );
}
