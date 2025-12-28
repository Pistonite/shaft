use cu::pre::*;

static LOGO: &'static str = r" ______ __  __ ______ ______ ______  
/\  ___\\ \_\ \\  __ \\  ___\\__  _\ 
\ \___  \\  __ \\  __ \\  __\/_/\ \/ 
 \/\_____\\_\ \_\\_\ \_\\_\    \ \_\  
  \/_____//_/\/_//_/\/_//_/     \/_/  ";

/// The component that keeps machine running
#[derive(clap::Parser, Debug, AsRef, Serialize, Deserialize)]
#[clap(
    version,
    before_help = LOGO
)]
pub struct CliApi {
    /// If a command was interrupted previously, discard it
    #[clap(short = 'A', long)]
    pub abort_previous: bool,
    #[clap(subcommand)]
    #[as_ref(cu::cli::Flags)]
    pub command: CliCommand,
}
impl CliApi {
    pub fn run(self) -> cu::Result<()> {
        cu::trace!("args: {self:#?}");
        op::init_platform()?;
        Ok(())
    }
}

#[derive(clap::Subcommand, Debug, Serialize, Deserialize)]
pub enum CliCommand {
    /// Upgrade this binary to the latest version
    Upgrade(#[serde(skip)] cu::cli::Flags),
    /// Install or update package(s)
    Sync(CliCommandSync),
    /// Remove package(s)
    Remove(CliCommandRemove),
    /// Edit configuration for a package
    Config(CliCommandConfig),
    /// Clean temporary files for this tool and/or package(s)
    Clean(CliCommandClean),
    /// Resume previous operation, if one was interrupted
    Resume(#[serde(skip)]cu::cli::Flags),
}
impl AsRef<cu::cli::Flags> for CliCommand {
    fn as_ref(&self) -> &cu::cli::Flags {
        match self {
            CliCommand::Upgrade(x) => x,
            CliCommand::Sync(x) => x.as_ref(),
            CliCommand::Remove(x) => x.as_ref(),
            CliCommand::Config(x) => x.as_ref(),
            CliCommand::Clean(x) => x.as_ref(),
            CliCommand::Resume(x) => x,
        }
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

#[derive(clap::Parser, Debug, AsRef, Serialize, Deserialize)]
pub struct CliCommandRemove {
    /// Package(s) to remove.
    pub packages: Vec<String>,
    #[clap(flatten)]
    #[as_ref]
    #[serde(skip)]
    pub flags: cu::cli::Flags,
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

#[derive(clap::Parser, Debug, AsRef, Serialize, Deserialize)]
pub struct CliCommandClean {
    /// Package(s) to clean. If none specified, will only clean this tool.
    pub package: Vec<String>,
    #[clap(flatten)]
    #[as_ref]
    #[serde(skip)]
    pub flags: cu::cli::Flags,
}
