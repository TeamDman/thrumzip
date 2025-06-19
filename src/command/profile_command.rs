use crate::command::GlobalArgs;
use clap::Args;
use clap::Subcommand;
use color_eyre::eyre::Result;
use super::profile_add_command::ProfileAddCommand;
use super::profile_list_command::ProfileListCommand;
use super::profile_show_command::ProfileShowCommand;
use super::profile_use_command::ProfileUseCommand;

#[derive(Args)]
pub struct ProfileCommand {
    #[clap(subcommand)]
    pub cmd: ProfileCommandInner,
}

#[derive(Subcommand)]
pub enum ProfileCommandInner {
    /// Initialize a new profile interactively
    Add,
    /// Show the current profile
    Show,
    /// Lists the available profiles
    List,
    /// Sets the active profile
    Use {
        /// Name of the profile to set as active
        name: Option<String>,
    }
}

impl ProfileCommand {
    pub async fn handle(self, global: GlobalArgs) -> Result<()> {
        match self.cmd {
            ProfileCommandInner::Add => ProfileAddCommand.handle(global).await,
            ProfileCommandInner::List => ProfileListCommand.handle(global).await,
            ProfileCommandInner::Show => ProfileShowCommand.handle(global).await,
            ProfileCommandInner::Use { name } => ProfileUseCommand.handle(global, name).await,
        }
    }
}
