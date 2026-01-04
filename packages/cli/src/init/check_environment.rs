use cu::pre::*;
use corelib::{opfs,hmgr};

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
mod windows {
    use std::io::ErrorKind;

    use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE};
    use winreg::RegKey;

    use super::*;
    pub fn check_init_environment(config: &Config) -> cu::Result<()> {
        if config.windows.control_personal_shell_folder {
            const KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders";
            let reg_key = 
            cu::check!(
                RegKey::predef(HKEY_CURRENT_USER).open_subkey(KEY_PATH),
                "failed to open user shell folder sub key")?;
            let value: String = match reg_key.get_value("Personal") {
                Ok(value) => value,
                Err(e) if e.kind() == ErrorKind::NotFound => "".to_string(),
                Err(e) => {
                    cu::rethrow!(e, "failed to get user shell folder value for 'Personal'");
                }
            };
            let expected_value = hmgr::paths::windows_shell_root();
            let expected_value_str = expected_value.as_utf8()?;
            if value != expected_value {
                cu::warn!("the current user shell folder is: '{value}', which is not managed by shaft");
                cu::hint!("  this is the location that stores powershell user profiles");
                cu::hint!("  set it to {expected_value_str} can reduce clutter in your other folders");
                if cu::yesno!("change the user shell folder to {expected_value_str}?")? {
                    // create the directory in case
                    cu::fs::make_dir(hmgr::paths::windows_shell_root())?;
                    if !opfs::is_sudo() {
                        cu::bailfyi!("setting registry requires sudo - please run `sudo shaft -vV`");
                    }
                    let reg_key = cu::check!(
                        RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(KEY_PATH, KEY_WRITE),
                        "failed to open user shell folder sub key for writing")?;
                    cu::check!(reg_key.set_value("Personal", &expected_value_str), "failed to set registry value")?;
                    cu::info!("shell folder set successfully");
                    cu::hint!("you may want to copy your current profiles over. They are usually at ~/Documents/WindowsPowerShell or ~/Documents/PowerShell (for PS7)");
                    cu::hint!("to ensure you check that, an error will be generated");
                    cu::bailfyi!("please check and copy over your current shell profiles if needed, then restart all terminal processes to reload the environment");
                } else {
                    let config_path = hmgr::paths::config_toml();
                    cu::hint!("you can disable this check permanently by setting `windows.control-personal-shell-folder` to false in '{}'", config_path.display());
                }
            }
        }

        // check HOME = %USERPROFILE% for max compatibility
        if config.windows.control_home {
            let user_profile = cu::env_var("USERPROFILE")?;
            let home = cu::check!(hmgr::windows::get_user("HOME"), "failed to get user home")?;
            if home != user_profile {
                cu::warn!("user profile is '{}'", user_profile);
                cu::warn!("the 'HOME' user environment variable is not set to %USERPROFILE%");
                cu::hint!("this may cause compatibility issue");
                if cu::yesno!("change HOME to %USERPROFILE% ?")? {
                    cu::check!(hmgr::windows::set_user("HOME", &user_profile), "failed to set user 'HOME'")?;
                }
            }
        }

        Ok(())
    }
}
