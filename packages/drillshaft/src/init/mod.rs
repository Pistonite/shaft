use std::path::PathBuf;

use cu::pre::*;
use drillshaft_env::EnvChangeReboot;

pub fn full_init() -> cu::Result<()> {
    drillshaft_env::init_platform()?;

    let home_path = cu::env_var("DRILLSHAFT_HOME")?;
    let home_path = if home_path.is_empty() {
        cu::warn!("DRILLSHAFT_HOME is not configured!");
        cu::warn!(
            "note that if you think it's already configured, DO NOT CONTINUE.\ninstead check why the environment variable is not being picked up."
        );
        if !cu::yesno!("do you want to set up DRILLSHAFT_HOME now?")? {
            cu::bail!("DRILLSHAFT_HOME setup cancelled");
        }
        let default_home = if cfg!(windows) {
            let dev_drive = cu::prompt!("if you want to set up DRILLSHAFT_HOME on a dev drive, enter the drive letter; otherwise press ENTER")?.to_ascii_uppercase();
            let default_home = if dev_drive.is_empty() {
                match std::env::home_dir() {
                    Some(mut x) => {
                        x.push(".config/pistonite-drillshaft");
                        x
                    }
                    // None => PathBuf::from("/opt/pistonite-drillshaft")
                    None => cu::bail!("failed to get user home"),
                }
            } else {
                PathBuf::from(format!("{dev_drive}:/.config/pistonite-drillshaft"))
            };
            default_home.normalize()?
        } else {
            match std::env::home_dir() {
                Some(mut x) => {
                    x.push(".config/pistonite-drillshaft");
                    x.normalize()?
                }
                None => PathBuf::from("/opt/pistonite-drillshaft"),
            }
        };

        cu::hint!(
            "the default DRILLSHAFT_HOME will be '{}'",
            default_home.display()
        );
        let user_input = cu::prompt!("press ENTER to accept the default, or enter another path")?;
        let user_selected_home = if user_input.is_empty() {
            default_home
        } else {
            user_input.into()
        };
        let user_selected_home = user_selected_home.normalize()?;
        let user_selected_home_utf8 = cu::check!(
            user_selected_home.as_utf8(),
            "the selected DRILLSHAFT_HOME is not utf-8"
        )?;
        if let Ok(false) = cu::fs::is_empty(&user_selected_home) {
            cu::bail!("selected DRILLSHAFT_HOME is a non-empty directory");
        }
        if !cu::yesno!("create DRILLSHAFT_HOME at '{user_selected_home_utf8}'?")? {
            cu::bail!("DRILLSHAFT_HOME setup cancelled");
        }
        cu::fs::make_dir_empty(&user_selected_home)?;

        cu::info!("DRILLSHAFT_HOME created.");

        cu::fs::write(
            user_selected_home.join("init.bash"),
            "export DRILLSHAFT_HOME=\"{user_selected_home_utf8}\"\nexport PATH=\"$DRILLSHAFT_HOME/bin:$PATH\"",
        )?;
        cu::fs::write(user_selected_home.join("init.pwsh"), "")?;
        // TODO - set windows env
        //
        if cfg!(windows) {
            cu::hint!(
                "please add the following to your powershell profile (`notepad $PROFILE.CurrentUserAllHosts`)\n\n# Drillshaft init script\n. $env:DRILLSHAFT_HOME\\init.pwsh\n"
            );
        } else {
            cu::hint!(
                "please add the following to your shell profile (replase .bash with your shell's suffix)\n\n# Drillshaft init script\n. {user_selected_home_utf8}/init.bash"
            );
        }
        let mut env_reboot =
            EnvChangeReboot::new(user_selected_home.join("env-change-reboot.json"))?;
        env_reboot.add("DRILLSHAFT_HOME", user_selected_home_utf8);
        env_reboot.write_and_bail()?;

        user_selected_home
    } else {
        PathBuf::from(home_path)
    };

    cu::check!(
        EnvChangeReboot::new(home_path.join("env-change-reboot.json"))?.check(),
        "env check failed - did you forget to restart the shell/terminal?"
    )?;

    op::home::init(home_path);

    Ok(())
}
