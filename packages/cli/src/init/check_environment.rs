use corelib::hmgr;
use cu::pre::*;

use crate::config::Config;

pub fn check_init_environment(_config: &Config) -> cu::Result<()> {
    hmgr::init_env()?;
    #[cfg(windows)]
    {
        windows::check_init_environment(_config)?;
    }
    Ok(())
}

#[cfg(windows)]
pub mod windows {
    use super::*;
    pub fn check_init_environment(config: &Config) -> cu::Result<()> {
        // check HOME = %USERPROFILE% for max compatibility
        if config.windows.control_home {
            let user_profile = cu::env_var("USERPROFILE")?;
            let home = cu::check!(hmgr::windows::get_user("HOME"), "failed to get user home")?;
            if home != user_profile {
                cu::warn!("user profile is '{}'", user_profile);
                cu::warn!("the 'HOME' user environment variable is not set to %USERPROFILE%");
                cu::hint!("this may cause compatibility issue");
                if cu::yesno!("change HOME to %USERPROFILE% ?")? {
                    cu::check!(
                        hmgr::windows::set_user("HOME", &user_profile),
                        "failed to set user 'HOME'"
                    )?;
                }
            }
        }

        Ok(())
    }
}
