use super::profile_command::ProfileCommand;
use super::sync_command::SyncCommand;
use super::validate_command::ValidateCommand;
use clap::Args;
use clap::Parser;
use clap::Subcommand;
use eyre::Result;

#[derive(Parser)]
#[clap(version)]
pub struct Command {
    #[clap(flatten)]
    pub global_args: GlobalArgs,
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// View and manage Thrumzip CLI profiles
    Profile(ProfileCommand),
    /// Synchronize active profile output directory with the active profile source directories
    Sync(SyncCommand),
    /// Audits the active profile destination directory for discrepencies with the zip file contents of the source directories
    Validate(ValidateCommand),
}

#[derive(Args)]
pub struct GlobalArgs {
    /// Enable debug logging
    #[clap(long)]
    pub debug: bool,
    /// Run in non-interactive mode
    #[clap(long)]
    pub non_interactive: bool,
}

impl Command {
    pub async fn handle(self) -> Result<()> {
        match self.command {
            Commands::Profile(cmd) => cmd.handle(self.global_args).await,
            Commands::Sync(cmd) => cmd.handle(self.global_args).await,
            Commands::Validate(cmd) => cmd.handle(self.global_args).await,
        }
    }
}
