use crate::command::GlobalArgs;
use crate::state::profiles::Profiles;
use eye_config::persistable_state::PersistableState;

pub struct ProfileListCommand;
impl ProfileListCommand {
    pub async fn handle(self, _global: GlobalArgs) -> eyre::Result<()> {
        let cfg = Profiles::load().await?;
        println!("Current profile:");
        println!("{}", Profiles::key().await?.file_path()?.display());
        println!("{cfg:#?}");
        Ok(())
    }
}
