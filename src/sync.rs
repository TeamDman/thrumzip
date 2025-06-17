use crate::command::GlobalArgs;
use crate::config_state::AppConfig;
use crate::count_files::count_files;
use crate::get_zips;
use crate::partition::Partition;
use crate::read_entries_from_zips;
use clap::Args;
use color_eyre::eyre::Result;
use color_eyre::eyre::WrapErr;
use eye_config::persistable_state::PersistableState;
use tracing::info;

#[derive(Args)]
pub struct SyncCommand;

impl SyncCommand {
    pub async fn handle(self, _global: GlobalArgs) -> Result<()> {
        // Load configuration
        let cfg = AppConfig::load()
            .await
            .wrap_err("Failed to load configuration")?;

        // Count existing destination files
        let dest_count = count_files(&cfg.destination).await;
        info!(
            "Found {} files and folders in the destination path",
            dest_count
        );

        // Gather zip files from sources
        let zips = get_zips::get_zips(&cfg).await?;
        info!("Found {} zip files in the source paths", zips.len());

        // Scan entries in parallel and build entry handles
        let entries = read_entries_from_zips::read_entries_from_zips(zips).await?;
        info!("Found {} entries in the source zips", entries.len());
        let entries = Partition::from(entries);
        info!(
            "Partitioned entries; {} unambiguous, {} ambiguous",
            entries.unambiguous_entries.len(),
            entries.ambiguous_entries.len()
        );

        for entry in entries.unambiguous_entries.values().take(10) {
            let dest = entry.splat_into_dir(&cfg.destination, false).await?;
            info!(
                "Synced: {}",
                dest.display()
            );
        }

        Ok(())
    }
}
