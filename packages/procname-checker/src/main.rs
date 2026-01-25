use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicU8;
use std::time::Duration;

use cu::pre::*;
use sysinfo::System;

#[derive(clap::Parser)]
struct Cli {
    arg1: String,
    args: Vec<String>,
    #[clap(flatten)]
    flags: cu::cli::Flags,
}

#[cu::cli(flags = "flags")]
fn main(cli: Cli) -> cu::Result<()> {
    let state = Arc::new(AtomicU8::new(0));

    // spawn a thread to refresh processes
    let thread = {
        let state = Arc::clone(&state);
        std::thread::spawn(move || {
            let mut system = System::new();
            let mut old_pids = HashSet::new();
            let mut can_print = false;
            loop {
                system.refresh_processes(
                    sysinfo::ProcessesToUpdate::All,
                    true, /* remove_dead */
                );
                let mut new_pids = HashSet::new();
                for (pid, process) in system.processes() {
                    new_pids.insert(*pid);
                    if old_pids.insert(*pid) {
                        let exe = match process.exe() {
                            Some(path) => path.display().to_string(),
                            None => "(none)".to_string(),
                        };
                        let name = process.name().display().to_string();
                        if can_print {
                            cu::info!("+[{pid}] [exe={exe}] (name: {name})");
                        }
                    }
                }
                old_pids = new_pids;
                let s = state.load(std::sync::atomic::Ordering::SeqCst);
                if s == 2 {
                    break;
                }
                if state
                    .compare_exchange_weak(
                        0,
                        1,
                        std::sync::atomic::Ordering::SeqCst,
                        std::sync::atomic::Ordering::SeqCst,
                    )
                    .is_ok()
                {
                    can_print = true;
                }
            }
        })
    };

    // wait for sysinfo to initialize
    while state.load(std::sync::atomic::Ordering::SeqCst) != 1 {
        std::thread::sleep(Duration::from_millis(20));
    }
    // spawn
    let exe = Path::new(&cli.arg1);
    exe.command().args(&cli.args).all_null().wait()?;

    state.store(2, std::sync::atomic::Ordering::SeqCst);
    let _ = thread.join();

    Ok(())
}
