use crate::command::GlobalArgs;
use crate::state::profiles::Profiles;
use eye_config::persistable_state::PersistableState;

pub struct ProfileListCommand;
impl ProfileListCommand {
    pub async fn handle(self, _global: GlobalArgs) -> eyre::Result<()> {
        let profiles = Profiles::load().await?;
        println!(
            "Current profile: {}",
            profiles.current().map_or("None".to_string(), |p| p.name.clone())
        );
        println!("{}", Profiles::key().await?.file_path()?.display());
        println!("{:#?}", profiles.profiles);
        Ok(())
    }
}
