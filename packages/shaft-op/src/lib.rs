mod platform;
pub use platform::*;
mod version;
pub use version::*;
pub mod installer;
/// Env modification checks
pub mod env_mod;
mod download;
pub use download::*;
mod main_thread;
pub use main_thread::*;

pub mod util;
pub mod home;
pub mod sysinfo;

mod init_impl {
    use cu::pre::*;
    /// Init all systems
    pub fn init() -> cu::Result<()> {
        // cu::check!(crate::sysinfo::init(), "failed to init sysinfo")?;
        Ok(())
    }
}
pub use init_impl::*;
