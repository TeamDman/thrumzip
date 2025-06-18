use crate::command::GlobalArgs;
use crate::config_init_command::ConfigInitCommand;
use crate::config_show_command::ConfigShowCommand;
use clap::Args;
use clap::Subcommand;
use color_eyre::eyre::Result;

#[derive(Args)]
pub struct ConfigCommand {
    #[clap(subcommand)]
    pub cmd: ConfigSub,
}

#[derive(Subcommand)]
pub enum ConfigSub {
    /// Initialize the configuration interactively
    Init,
    /// Show the current configuration
    Show,
}

impl ConfigCommand {
    pub async fn handle(self, global: GlobalArgs) -> Result<()> {
        match self.cmd {
            ConfigSub::Init => ConfigInitCommand.handle(global).await,
            ConfigSub::Show => ConfigShowCommand.handle(global).await,
        }
    }
}
