use clap::Args;
use clap::Parser;
use clap::Subcommand;
use eyre::Result;

use super::config_command::ConfigCommand;
use super::sync::SyncCommand;

#[derive(Parser)]
#[clap(name = "meta-takeout", version)]
pub struct Command {
    #[clap(flatten)]
    pub global_args: GlobalArgs,
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Config(ConfigCommand),
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
            Commands::Config(cmd) => cmd.handle(self.global_args).await,
            Commands::Sync(cmd) => cmd.handle(self.global_args).await,
        }
    }
}
