use super::profile_command::ProfileCommand;
use super::sync::SyncCommand;
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
        }
    }
}
