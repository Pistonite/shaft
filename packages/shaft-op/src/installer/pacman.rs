use std::{collections::BTreeSet, sync::{Mutex, MutexGuard, atomic::{self, AtomicBool}}};

use cu::pre::*;

static STATE: Mutex<State> = Mutex::new(State::new());
struct State {
    installed_packages: BTreeSet<String>,
}
impl State {
    pub const fn new() -> Self {
        Self {
            installed_packages: BTreeSet::new(),
        }
    }
}
static USING: AtomicBool = AtomicBool::new(false);

/// Lock pacman state for access the entirety of pacman
pub fn lock() -> cu::Result<PacmanGuard> {
    if USING.compare_exchange(false, true, atomic::Ordering::SeqCst, atomic::Ordering::SeqCst).is_err() {
        cu::bail!("pacman::lock can only be called from main installation thread");
    }
    // unwrap: another thread already panicked, we will panic too
    Ok(PacmanGuard(STATE.lock().unwrap()))
}

pub struct PacmanGuard(MutexGuard<'static, State>);
impl Drop for PacmanGuard {
    fn drop(&mut self) {
        // it's definitely `true` if PacmanGuard exists, so we don't need to compare_exchange
        USING.store(false, atomic::Ordering::SeqCst);
    }
}
impl PacmanGuard {
    /// Check if a package is installed with pacman
    pub fn is_installed(&mut self, package_name: &str) -> cu::Result<bool> {
        if self.0.installed_packages.is_empty() {
            cu::debug!("pacman: querying installed packages");
            let (child, stdout) = cu::which("pacman")?
                .command()
                .arg("-Qq")
                .stdout(cu::pio::string())
                .stdie_null()
                .spawn()?;
            child.wait_nz()?;
            let stdout = stdout.join()??;
            self.0
                .installed_packages
                .extend(stdout.lines().map(|x| x.trim().to_string()))
        }
        Ok(self.0.installed_packages.contains(package_name))
    }
}


