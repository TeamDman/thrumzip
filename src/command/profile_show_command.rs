use crate::command::GlobalArgs;
use crate::state::profiles::Profiles;
use eye_config::persistable_state::PersistableState;
use eyre::OptionExt;

pub struct ProfileShowCommand;
impl ProfileShowCommand {
    pub async fn handle(self, _global: GlobalArgs) -> eyre::Result<()> {
        let profiles = Profiles::load().await?;
        let current_profile = profiles
            .current()
            .ok_or_eyre("No active profile found. Please set an active profile first.")?;
        eprintln!("Current Profile:");
        println!("{}", serde_json::to_string_pretty(current_profile).unwrap_or_else(|_| "Failed to serialize profile".to_string()));
        Ok(())
    }
}
