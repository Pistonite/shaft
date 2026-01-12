use std::path::{Path, PathBuf};

use corelib::ItemMgr;
use cu::pre::*;

use corelib::hmgr;

pub fn check_init_home() -> cu::Result<()> {
    let home_path_str = cu::env_var("SHAFT_HOME")?;
    let home_path = Path::new(&home_path_str).normalize()?;
    if !home_path_str.is_empty() && home_path.is_dir() {
        hmgr::paths::init_home_path(home_path);
        return Ok(());
    }

    if !home_path_str.is_empty() {
        cu::warn!("did not find home at: '{}'", home_path.display());
        if !cu::yesno!(
            "do you want to try creating an empty directory here as the home for this tool?"
        )? {
            cu::bail!("SHAFT_HOME does not point to an existing directory");
        }
        cu::check!(
            cu::fs::make_dir(&home_path),
            "failed to create home directory"
        )?;
        cu::info!("home directory created!");
        // re-normalize since it didn't exist before
        hmgr::paths::init_home_path(home_path.normalize()?);
        return Ok(());
    }

    cu::error!("SHAFT_HOME not set!");
    cu::hint!(
        "if this is the first time running the tool, please follow the prompts to initialize."
    );
    cu::warn!(
        "if you already initialized the tool, make sure you have added the required initialization scripts to the shell profile"
    );
    if !cu::yesno!("do you want to initialize the tool now")? {
        cu::bail!("SHAFT_HOME not set, please follow the prompts to initialize the tool");
    }

    let default_home = if cfg!(windows) {
        cu::hint!(
            r"there may be performance benefit to install dev tools on a Dev Drive.
read more at: https://learn.microsoft.com/en-us/windows/dev-drive/

if the dev drive is setup as a virtual disk (.vhdx), restart the computer a few times to ensure 
it can be reliably auto-mounted on system start. Sometimes auto-mount can fail for SATA drives,
the workaround is put the .vhdx on the OS drive. You can use Event Viewer to inspect Kernel-IO errors
to see why the auto-mount fails.
        "
        );
        let dev_drive = cu::prompt!("if you want to set up SHAFT_HOME on a Windows Dev Drive, enter the drive letter; otherwise press ENTER")?.to_ascii_uppercase();
        let default_home = if dev_drive.is_empty() {
            match std::env::home_dir() {
                Some(mut x) => {
                    x.push(".config\\pistonite-shaft");
                    x
                }
                None => cu::bail!("failed to get user home"),
            }
        } else {
            PathBuf::from(format!("{dev_drive}:\\.config\\pistonite-shaft"))
        };
        default_home.normalize()?
    } else {
        match std::env::home_dir() {
            Some(mut x) => {
                x.push(".config/pistonite-shaft");
                x.normalize()?
            }
            None => PathBuf::from("/opt/pistonite-shaft"),
        }
    };

    let home = loop {
        match prompt_user_input_for_home(&default_home) {
            Err(e) => {
                cu::error!("{e:?}");
                continue;
            }
            Ok(x) => {
                break x;
            }
        }
    };
    if !cu::yesno!("create home directory at '{}'?", home.display())? {
        cu::bail!("setup cancelled");
    }

    let bar = cu::progress("initializing home").spawn();
    cu::progress!(bar, "creating home directory");
    cu::check!(
        cu::fs::make_dir_empty(&home),
        "failed to create home directory"
    )?;
    cu::info!("home directory created!");

    // renormalize after creation, since it could be different
    let home = home.normalize()?;
    hmgr::paths::init_home_path(home.clone());
    let home_str = home.as_utf8()?;

    cu::progress!(bar, "initializing items");
    let mut items = ItemMgr::load()?;
    items.skip_reinvocation(true);
    items.rebuild_items(Some(&bar))?;
    bar.done();

    if cfg!(windows) {
        hmgr::windows::set_user("SHAFT_HOME", home_str)?;
        let control_shell_folder =
            crate::init::check_environment::windows::check_user_shell_folder()?;
        let mut added = false;
        cu::info!("checking init script...");
        let init_script = r#"# shaft init script
. $env:SHAFT_HOME\items\init.ps1
"#;
        if control_shell_folder {
            if cu::yesno!("add the init script to the powershell profile?")? {
                let root = hmgr::paths::windows_shell_root();
                let ps5 = root.join("WindowsPowerShell\\profile.ps1");
                let content = cu::fs::read_string(&ps5).unwrap_or_default();
                let content = format!("{init_script}\n{content}");
                cu::fs::write(ps5, content)?;
                let ps7 = root.join("PowerShell\\profile.ps1");
                let content = cu::fs::read_string(&ps7).unwrap_or_default();
                let content = format!("{init_script}\n{content}");
                cu::fs::write(ps7, content)?;
                added = true;
            }
        }
        if !added {
            cu::hint!("ATTENTION! please add the following to your powershell profile:");
            println!("\n{}\n", init_script);
            cu::hint!("you can open the profile by `notepad $PROFILE.CurrentUserAllHosts`");
            cu::prompt!("please press ENTER to continue once it's added")?;
        }
    } else {
        let init_script = format!(
            r#"# shaft init script
. {home_str}/items/init.bash
"#
        );
        cu::hint!("ATTENTION! please add the following to your shell profile:");
        println!("\n{}\n", init_script);
        cu::hint!("you can replace `.bash` with the shell you use");
        cu::prompt!("please press ENTER to continue once it's added")?;
    }
    hmgr::add_env_assert([("SHAFT_HOME".to_string(), home_str.to_string())])?;
    hmgr::require_envchange_reinvocation(false)
}

fn prompt_user_input_for_home(default_home: &Path) -> cu::Result<PathBuf> {
    cu::hint!(
        "the default SHAFT_HOME will be '{}'",
        default_home.display()
    );
    let user_input = cu::prompt!("press ENTER to accept the default, or enter another path")?;
    let user_selected_home = if user_input.is_empty() {
        default_home.to_path_buf()
    } else {
        user_input.into()
    };
    let user_selected_home = user_selected_home.normalize()?;
    cu::check!(
        user_selected_home.as_utf8(),
        "the selected SHAFT_HOME is not utf-8"
    )?;

    if let Ok(false) = cu::fs::is_empty_dir(&user_selected_home) {
        cu::bail!("selected SHAFT_HOME is a non-empty directory, please select another location");
    }

    Ok(user_selected_home)
}
