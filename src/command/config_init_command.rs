use crate::command::GlobalArgs;
use crate::config_state::AppConfig;
use crate::config_state::AppProfile;
use crate::config_state::DEFAULT_IMAGE_SIMILARITY_THRESHOLD;
use cloud_terrastodon_user_input::prompt_line;
use color_eyre::eyre::Result;
use color_eyre::eyre::WrapErr;
use color_eyre::eyre::bail;
use eye_config::persistable_state::PersistableState;

pub struct ConfigInitCommand;
impl ConfigInitCommand {
    pub async fn handle(&self, global: GlobalArgs) -> Result<()> {
        if global.non_interactive {
            bail!("Cannot initialize config in non-interactive mode");
        }
        // Load existing or default config
        let mut config = AppConfig::load()
            .await
            .wrap_err("Failed to load existing config")?;

        // Prompt the user for the new profile details
        let name = prompt_line("Enter the name of the new config profile: ")
            .await
            .wrap_err("Failed to read profile name")?;
        if config.profiles.iter().any(|p| p.name == name) {
            bail!("A profile with the name '{}' already exists", name);
        }

        let destination = prompt_line("Enter the path to the destination directory: ")
            .await
            .wrap_err("Failed to read destination path")?;

        let sources = {
            let mut sources = Vec::new();
            loop {
                let src = prompt_line("Enter a source directory (empty to finish): ")
                    .await
                    .wrap_err("Failed to read source directory")?;
                let src = src.trim().trim_matches('"');
                if src.is_empty() {
                    break;
                }
                sources.push(src.to_string().into());
            }
            sources
        };

        let similarity = {
            let similarity = prompt_line(&format!(
                "Enter the similarity threshold for images [{}]: ",
                DEFAULT_IMAGE_SIMILARITY_THRESHOLD
            ))
            .await
            .wrap_err("Failed to read similarity threshold")?;
            let similarity = similarity.trim();
            let similarity = if !similarity.is_empty() {
                similarity.parse().wrap_err("Invalid similarity value")?
            } else {
                DEFAULT_IMAGE_SIMILARITY_THRESHOLD
            };
            similarity
        };

        // Push the new profile to the config
        config.profiles.push(AppProfile {
            destination: destination.into(),
            sources,
            similarity,
            name,
        });
        
        // Save config
        config.save().await.wrap_err("Failed to save config")?;

        // Show location
        let path = AppConfig::key().await?.file_path()?;
        println!("Configuration saved to {}", path.display());
        
        Ok(())
    }
}
