use crate::command::GlobalArgs;
use crate::config_state::AppConfig;
use eye_config::persistable_state::PersistableState;
use eyre::Context;

pub struct ConfigShowCommand;
impl ConfigShowCommand {
    pub async fn handle(self, _global: GlobalArgs) -> eyre::Result<()> {
        // Load configuration
        let cfg = AppConfig::load()
            .await
            .wrap_err("Failed to load configuration")?;

        // Display the current configuration
        println!("Current Configuration:");
        println!("{}", AppConfig::key().await?.file_path()?.display());
        println!("{cfg:#?}");

        Ok(())
    }
}
