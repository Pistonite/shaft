use std::path::Path;

use corelib::{hmgr, opfs};
use cu::pre::*;

static LOGO: &'static str = r" ______ __  __ ______ ______ ______  
/\  ___\\ \_\ \\  __ \\  ___\\__  _\ 
\ \___  \\  __ \\  __ \\  __\/_/\ \/ 
 \/\_____\\_\ \_\\_\ \_\\_\    \ \_\  
  \/_____//_/\/_//_/\/_//_/     \/_/  ";

/// The component that keeps machine running
#[derive(clap::Parser, Debug, AsRef, Serialize, Deserialize)]
#[clap(before_help = LOGO)]
pub struct CliApi {
    /// If a command was interrupted previously, discard it
    #[clap(short = 'A', long)]
    pub abort_previous: bool,
    #[clap(subcommand)]
    pub command: Option<CliCommand>,
    #[as_ref]
    #[serde(skip)]
    #[clap(flatten)]
    flags: cu::cli::Flags,

    /// Same as the version subcommand, -v to run self-check
    #[clap(short = 'V', long)]
    version: bool,
}
impl CliApi {
    pub fn preprocess(&mut self) {
        if let Some(command) = &self.command {
            self.flags.merge(command.as_ref());
        }
    }
    pub fn run(self) -> cu::Result<()> {
        let run_version = self.version || matches!(&self.command, Some(CliCommand::Version(_)));
        if run_version {
            cu::lv::disable_print_time();
            println!("{}", clap::crate_version!());
            if !cu::lv::D.enabled() {
                return Ok(());
            }
        }

        // unless running 'version', do not allow sudo
        if !run_version {
            if opfs::is_sudo() {
                cu::bail!(
                    "do not run shaft as root, privilege will be escalated automatically if needed."
                );
            }
        }
        cu::trace!("args: {self:#?}");
        cu::check!(
            corelib::check_requirements(),
            "core requirements not satisfied"
        )?;
        cu::check!(
            opfs::init(clap::crate_version!()),
            "failed to init platform"
        )?;
        cu::check!(crate::init::check_init_home(), "failed to init home")?;
        let config = crate::config::load_config()?;
        cu::check!(
            crate::init::check_init_environment(&config),
            "failed to init environment"
        )?;

        // this is to make it easier to run the tool in development
        #[cfg(not(debug_assertions))]
        {
            cu::check!(crate::init::check_init_binary(), "failed to init binary")?;
        }

        if run_version {
            cu::info!("self-check OK");
            return Ok(());
        }

        let Some(command) = self.command else {
            cu::lv::disable_print_time();
            cu::cli::print_help::<Self>(false);
            return Ok(());
        };

        let _lock = hmgr::lock()?;

        let previous = hmgr::extract_previously_interrupted_json_command();
        if matches!(&command, CliCommand::Resume(_)) {
            if self.abort_previous {
                if previous.is_some() {
                    cu::info!("aborted previous command; nothing to resume");
                } else {
                    cu::warn!("no previous command to abort");
                }
            } else {
                if let Some((cmd_args, cmd_str)) = previous {
                    let previous_command = cu::check!(
                        json::parse::<CliCommand>(&cmd_str),
                        "previous command file is corrupted"
                    )?;
                    cu::info!("resuming: {cmd_args}");
                    previous_command.run()?;
                } else {
                    cu::warn!("no previous command to resume");
                }
            }
            return Ok(());
        }
        if let Some((cmd_args, cmd_str)) = previous {
            match json::parse::<CliCommand>(&cmd_str) {
                Err(e) => {
                    cu::error!("failed to parse previous command: {e:?}");
                    cu::warn!("ignoring corrupted previous command file");
                }
                Ok(previous_command) => {
                    cu::warn!("found previously interrupted command:\n  {cmd_args}");
                    cu::hint!(
                        "the command can be resumed.\n- Y = execute previous command, then execute current command\n- N = only execute current command, discard previous command"
                    );
                    if cu::yesno!("resume previous command?")? {
                        cu::info!("resuming: {cmd_args}");
                        previous_command.run()?;
                    }
                }
            }
        }

        command.run()
    }
}

#[derive(clap::Subcommand, Debug, Serialize, Deserialize)]
pub enum CliCommand {
    /// Upgrade this binary to the latest version
    Upgrade(CliCommandUpgrade),
    /// Install or update package(s)
    Sync(CliCommandSync),
    /// Remove package(s)
    Remove(CliCommandRemove),
    /// Edit configuration for a package
    Config(CliCommandConfig),
    /// Clean temporary files for this tool and/or package(s)
    Clean(CliCommandClean),
    /// Resume previous operation, if one was interrupted
    Resume(#[serde(skip)] cu::cli::Flags),
    /// Print the version, -v to run self-check
    Version(#[serde(skip)] cu::cli::Flags),
}
impl AsRef<cu::cli::Flags> for CliCommand {
    fn as_ref(&self) -> &cu::cli::Flags {
        match self {
            CliCommand::Upgrade(x) => x.as_ref(),
            CliCommand::Sync(x) => x.as_ref(),
            CliCommand::Remove(x) => x.as_ref(),
            CliCommand::Config(x) => x.as_ref(),
            CliCommand::Clean(x) => x.as_ref(),
            CliCommand::Resume(x) => x,
            CliCommand::Version(x) => x,
        }
    }
}
impl CliCommand {
    pub fn run(self) -> cu::Result<()> {
        if !matches!(self, CliCommand::Version(_) | CliCommand::Resume(_)) {
            match json::stringify_pretty(&self) {
                Err(e) => {
                    cu::error!("failed to stringify command: {e:?}");
                }
                Ok(s) => {
                    hmgr::save_command_json(&s);
                }
            }
        }
        match self {
            CliCommand::Version(_) => {}
            CliCommand::Resume(_) => {}
            CliCommand::Upgrade(cmd) => cmd.run()?,
            CliCommand::Sync(cmd) => cmd.run()?,
            CliCommand::Remove(cmd) => cmd.run()?,
            CliCommand::Config(cmd) => cmd.run()?,
            CliCommand::Clean(_) => {}
        }
        Ok(())
    }
}

#[derive(clap::Parser, Debug, AsRef, Serialize, Deserialize)]
pub struct CliCommandUpgrade {
    /// Build and install from this path, instead of from upstream
    #[clap(short, long)]
    pub path: Option<String>,
    #[clap(flatten)]
    #[as_ref]
    #[serde(skip)]
    pub flags: cu::cli::Flags,
}

impl CliCommandUpgrade {
    fn run(&self) -> cu::Result<()> {
        crate::cmds::upgrade(self.path.as_ref().map(Path::new))
    }
}

#[derive(clap::Parser, Debug, AsRef, Serialize, Deserialize)]
pub struct CliCommandSync {
    /// Package(s) to install or update. If none specified, will update all installed packages.
    pub packages: Vec<String>,
    #[clap(flatten)]
    #[as_ref]
    #[serde(skip)]
    pub flags: cu::cli::Flags,
}
impl CliCommandSync {
    fn run(&self) -> cu::Result<()> {
        crate::cmds::sync(&self.packages)
    }
}

#[derive(clap::Parser, Debug, AsRef, Serialize, Deserialize)]
pub struct CliCommandRemove {
    /// Package(s) to remove.
    pub packages: Vec<String>,
    #[clap(flatten)]
    #[as_ref]
    #[serde(skip)]
    pub flags: cu::cli::Flags,
}
impl CliCommandRemove {
    fn run(&self) -> cu::Result<()> {
        crate::cmds::remove(&self.packages)
    }
}

#[derive(clap::Parser, Debug, AsRef, Serialize, Deserialize)]
pub struct CliCommandConfig {
    /// Package to config
    pub package: String,
    /// Print config location instead of opening.
    #[clap(short, long)]
    pub location: bool,
    #[clap(flatten)]
    #[as_ref]
    #[serde(skip)]
    pub flags: cu::cli::Flags,
}
impl CliCommandConfig {
    fn run(&self) -> cu::Result<()> {
        cu::lv::disable_print_time();
        if self.location {
            let location = crate::cmds::config_location(&self.package)?;
            println!("{}", location);
            return Ok(());
        }
        crate::cmds::config(&self.package)
    }
}

#[derive(clap::Parser, Debug, AsRef, Serialize, Deserialize)]
pub struct CliCommandClean {
    /// Package(s) to clean. If none specified, will only clean this tool.
    pub package: Vec<String>,
    #[clap(flatten)]
    #[as_ref]
    #[serde(skip)]
    pub flags: cu::cli::Flags,
}
