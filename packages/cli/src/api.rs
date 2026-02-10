use corelib::{hmgr, opfs};
use cu::pre::*;

static LOGO: &str = r" ______ __  __ ______ ______ ______  
/\  ___\\ \_\ \\  __ \\  ___\\__  _\ 
\ \___  \\  __ \\  __ \\  __\/_/\ \/ 
 \/\_____\\_\ \_\\_\ \_\\_\    \ \_\  
  \/_____//_/\/_//_/\/_//_/     \/_/  ";

/// The component that keeps the machine running
#[derive(clap::Parser, Debug, AsRef)]
#[clap(before_help = LOGO)]
pub struct CliApi {
    /// If a command was interrupted previously, discard it
    #[clap(short = 'A', long)]
    pub abort_previous: bool,
    #[clap(subcommand)]
    pub command: Option<CliCommand>,
    #[as_ref]
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
            println!("{}", env!("CARGO_PKG_VERSION"));
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
            opfs::init(env!("CARGO_PKG_VERSION")),
            "failed to init platform"
        )?;
        cu::check!(crate::init::check_init_home(), "failed to init home")?;
        let config = crate::config::load_config()?;
        cu::check!(
            crate::init::check_init_environment(&config),
            "failed to init environment"
        )?;

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

        command.run()
    }
}

#[derive(clap::Subcommand, Debug)]
pub enum CliCommand {
    /// Upgrade this binary to the latest version
    Upgrade(CliCommandUpgrade),
    /// Install or update package(s)
    Sync(CliCommandSync),
    /// Remove package(s)
    Remove(CliCommandRemove),
    /// Edit configuration for a package
    Config(CliCommandConfig),
    /// Search or print info of a package or binary
    Info(CliCommandInfo),
    /// Clean temporary files for this tool and/or package(s)
    Clean(CliCommandClean),
    /// Print the version, -v to run self-check
    Version(cu::cli::Flags),
}
impl AsRef<cu::cli::Flags> for CliCommand {
    fn as_ref(&self) -> &cu::cli::Flags {
        match self {
            CliCommand::Upgrade(x) => x.as_ref(),
            CliCommand::Sync(x) => x.as_ref(),
            CliCommand::Remove(x) => x.as_ref(),
            CliCommand::Config(x) => x.as_ref(),
            CliCommand::Clean(x) => x.as_ref(),
            CliCommand::Info(x) => x.as_ref(),
            CliCommand::Version(x) => x,
        }
    }
}
impl CliCommand {
    pub fn run(self) -> cu::Result<()> {
        match self {
            CliCommand::Version(_) => {}
            CliCommand::Upgrade(cmd) => cmd.run()?,
            CliCommand::Sync(cmd) => cmd.run()?,
            CliCommand::Remove(cmd) => cmd.run()?,
            CliCommand::Config(cmd) => cmd.run()?,
            CliCommand::Info(cmd) => cmd.run()?,
            CliCommand::Clean(_) => {}
        }
        Ok(())
    }
}

#[derive(clap::Parser, Debug, AsRef)]
pub struct CliCommandUpgrade {
    #[clap(flatten)]
    #[as_ref]
    pub flags: cu::cli::Flags,
}

impl CliCommandUpgrade {
    fn run(&self) -> cu::Result<()> {
        corelib::hmgr::repo::local_update()
    }
}

#[derive(clap::Parser, Debug, AsRef)]
pub struct CliCommandSync {
    /// Package(s) to install or update. If none specified, will update all installed packages.
    pub packages: Vec<String>,
    #[clap(flatten)]
    #[as_ref]
    pub flags: cu::cli::Flags,
}
impl CliCommandSync {
    fn run(&self) -> cu::Result<()> {
        crate::cmds::sync(&self.packages)
    }
}

#[derive(clap::Parser, Debug, AsRef)]
pub struct CliCommandRemove {
    /// Package(s) to remove.
    pub packages: Vec<String>,
    /// Force uninstall when package is in an unclean state.
    #[clap(short, long)]
    pub force: bool,
    #[clap(flatten)]
    #[as_ref]
    pub flags: cu::cli::Flags,
}
impl CliCommandRemove {
    fn run(&self) -> cu::Result<()> {
        crate::cmds::remove(&self.packages, self.force)
    }
}

#[derive(clap::Parser, Debug, AsRef)]
pub struct CliCommandConfig {
    /// Package to config
    pub package: Option<String>,
    /// Print config location instead of opening.
    #[clap(short, long)]
    pub location: bool,
    /// Just mark the config as dirty instead of editing
    #[clap(short, long)]
    pub dirty: bool,
    #[clap(flatten)]
    #[as_ref]
    pub flags: cu::cli::Flags,
}
impl CliCommandConfig {
    fn run(&self) -> cu::Result<()> {
        cu::lv::disable_print_time();
        if self.location {
            let Some(package) = &self.package else {
                cu::bail!("please specify a package name");
            };
            let location = crate::cmds::config_location(package)?;
            println!("{}", location);
            return Ok(());
        }
        if self.dirty {
            match &self.package {
                None => crate::cmds::config_dirty_all(),
                Some(package) => crate::cmds::config_dirty(package),
            }
        } else {
            let Some(package) = &self.package else {
                cu::bail!("please specify a package name");
            };
            crate::cmds::config(package)
        }
    }
}

#[derive(clap::Parser, Debug, AsRef)]
pub struct CliCommandInfo {
    /// Package or binary. Use --search to search. Must be provided if --installed is false
    ///
    /// Specify --binary or --package to narrow the type to search
    pub pkg_or_bin_query: Option<String>,

    /// Treat the pkg_or_bin input as string to search rather than exact match
    #[clap(short, long)]
    pub search: bool,

    /// Treat the pkg_or_bin as name of a binary
    #[clap(short, long, conflicts_with = "package")]
    pub binary: bool,

    /// Treat the pkg_or_bin as name of a package
    #[clap(short, long, conflicts_with = "binary")]
    pub package: bool,

    /// Scope results to installed packages only
    #[clap(short = 'I', long)]
    pub installed: bool,

    /// Machine mode. Only print the name of the package, one per line
    #[clap(short, long)]
    pub machine: bool,

    #[clap(flatten)]
    #[as_ref]
    pub flags: cu::cli::Flags,
}
impl CliCommandInfo {
    fn run(self) -> cu::Result<()> {
        if self.machine {
            cu::lv::disable_print_time();
        }
        let result = crate::cmds::info(
            self.pkg_or_bin_query.as_deref().unwrap_or_default(),
            self.search,
            self.installed,
            self.binary,
            self.package,
            self.machine,
        );
        let found = cu::check!(result, "error getting package information")?;
        if !found {
            cu::bail!("no results");
        }
        Ok(())
    }
}

#[derive(clap::Parser, Debug, AsRef)]
pub struct CliCommandClean {
    /// Package(s) to clean. If none specified, will only clean this tool.
    pub package: Vec<String>,
    #[clap(flatten)]
    #[as_ref]
    pub flags: cu::cli::Flags,
}
