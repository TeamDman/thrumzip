use crate::command::GlobalArgs;
use crate::config_state::AppConfig;
use color_eyre::eyre::Result;
use color_eyre::eyre::WrapErr;
use color_eyre::eyre::bail;
use eye_config::persistable_state::PersistableState;
use std::io::Write;
use std::io::{self};
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
        let mut stdout = io::stdout();
        // Destination path
        print!("Enter the path to the destination directory: ");
        stdout.flush()?;
        let mut dest = String::new();
        io::stdin().read_line(&mut dest)?;
        let dest = dest.trim().trim_matches('"').to_string();
        config.destination = dest.into();
        // Sources
        let mut sources = Vec::new();
        loop {
            print!("Enter a source directory (empty to finish): ");
            stdout.flush()?;
            let mut src = String::new();
            io::stdin().read_line(&mut src)?;
            let src = src.trim().trim_matches('"');
            if src.is_empty() {
                break;
            }
            sources.push(src.to_string().into());
        }
        config.sources = sources;
        // Similarity threshold
        print!(
            "Enter the similarity threshold for images [{}]: ",
            config.similarity
        );
        stdout.flush()?;
        let mut sim = String::new();
        io::stdin().read_line(&mut sim)?;
        let sim = sim.trim();
        if !sim.is_empty() {
            config.similarity = sim.parse().wrap_err("Invalid similarity value")?;
        }
        // Save config
        config.save().await.wrap_err("Failed to save config")?;
        // Show location
        let path = AppConfig::key().await?.file_path()?;
        println!("Configuration saved to {}", path.display());
        Ok(())
    }
}
