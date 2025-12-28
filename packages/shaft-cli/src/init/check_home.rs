use std::path::{Path, PathBuf};

use cu::pre::*;

pub fn check_init_home() -> cu::Result<()> {
    let home_path_str = cu::env_var("SHAFT_HOME")?;
    let home_path = Path::new(&home_path_str).normalize()?;
    if !home_path_str.is_empty() && home_path.is_dir() {
        op::home::init(home_path);
        return Ok(());
    }

    if !home_path_str.is_empty() {
        cu::warn!("did not find home at: '{}'", home_path.display());
        if !cu::yesno!("do you want to try creating an empty directory here as the home for this tool?")? {
            cu::bail!("SHAFT_HOME does not point to an existing directory");
        }
        cu::check!(cu::fs::make_dir(&home_path), "failed to create home directory")?;
        cu::info!("home directory created!");
        // re-normalize since it didn't exist before
        op::home::init(home_path.normalize()?);
        return Ok(());
    }

    cu::warn!("SHAFT_HOME not set!");
    cu::warn!("if this is the first time running the tool, please follow the prompts to initialize.");
    cu::warn!("if you already initialized the tool, make sure you have added the required initialization scripts to the shell profile");
    if !cu::yesno!("do you want to initialize the tool now")? {
        cu::bail!("SHAFT_HOME not set, please follow the prompts to initialize the tool");
    }

    let default_home = if cfg!(windows) {
        cu::hint!("there may be performance benefit to install dev tools on a Dev Drive.");
        cu::hint!("read more at: https://learn.microsoft.com/en-us/windows/dev-drive/");
        let dev_drive = cu::prompt!("if you want to set up SHAFT_HOME on a Windows Dev Drive, enter the drive letter; otherwise press ENTER")?.to_ascii_uppercase();
        let default_home = if dev_drive.is_empty() {
            match std::env::home_dir() {
                Some(mut x) => {
                    x.push(".config/pistonite-shaft");
                    x
                }
                None => cu::bail!("failed to get user home"),
            }
        } else {
            PathBuf::from(format!("{dev_drive}:/.config/pistonite-shaft"))
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
    cu::check!(cu::fs::make_dir_empty(&home), "failed to create home directory")?;
    cu::info!("home directory created!");

    op::home::init(home.normalize()?);

    let init_dir = home.join("init");
    create_init_bash(&home, &init_dir)?;
    create_init_pwsh(&home, &init_dir)?;
    cu::info!("init scripts created!");
    if cfg!(windows) {
        // CurrentUserAllHosts is for all hosts that run powershell, (for example, different
        // terminals, VS Code, etc...
        cu::hint!(
        r"please add the following to your powershell profile (`notepad $PROFILE.CurrentUserAllHosts`)

# shaft init script
. $env:SHAFT_HOME\init\init.pwsh
"
    );

        // we only need 
    } else {
        cu::hint!(
            r"please add the following to your (bash) profile

# shaft init script
. {}/init/init.pwsh
", home.as_utf8()?
        );
    }



        if let Ok(false) = cu::fs::is_empty_dir(&user_selected_home) {
            cu::bail!("selected SHAFT_HOME is a non-empty directory");
        }
    todo!()
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

fn create_init_bash(home: &Path, init_dir: &Path) -> cu::Result<()> {
    let init_bash = init_dir.join("init.bash");
    let content = format!(r#"# init/init.bash
# this file is managed by the tool, do not edit manually
export SHAFT_HOME="{}"
export PATH="$SHAFT_HOME:$SHAFT_HOME/bin:$PATH"
# ===
    "#, home.as_utf8()?);
    cu::fs::write(init_bash, content)?;
    Ok(())
}

fn create_init_pwsh(_: &Path, init_dir: &Path) -> cu::Result<()> {
    let init_pwsh = init_dir.join("init.pwsh");
    let content = r#"# init/init.pwsh
# this file is managed by the tool, do not edit manually
# ===
    "#;
    cu::fs::write(init_pwsh, content)?;
    Ok(())
}
