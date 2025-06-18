use crate::command::GlobalArgs;
use crate::config_state::AppConfig;
use crate::count_files::count_files;
use crate::get_zips;
use crate::partition::PartitionStrategy;
use crate::partition_strategy_unique_crc32::UniqueCrc32HashPartitionStrategy;
use crate::partition_strategy_unique_name::UniqueNamePartitionStrategy;
use crate::read_entries_from_zips;
use crate::sync_unambiguous_entries::sync_unambiguous_entries;
use clap::Args;
use color_eyre::eyre::Result;
use color_eyre::eyre::WrapErr;
use eye_config::persistable_state::PersistableState;
use eyre::bail;
use itertools::Itertools;
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
        info!("Crawling destination path: {}", cfg.destination.display());
        let dest_count = count_files(&cfg.destination).await;
        info!(
            "Found {} existing files in the destination path",
            dest_count
        );

        // Gather zip files from sources
        let zips = get_zips::get_zips(&cfg).await?;
        info!("Found {} zip files in the source paths", zips.len());
        // Scan entries in parallel
        let entries = read_entries_from_zips::read_entries_from_zips(zips).await?;
        // Log the number of entries found
        info!("Found {} entries in the source zips", entries.len());

        // Partition entries by unique names
        let partition = UniqueNamePartitionStrategy::partition(entries);
        info!(
            "Partitioned entries; {} unambiguous, {} ambiguous by name",
            partition.unambiguous_entries.len(),
            partition.ambiguous_entries.len()
        );

        // Sync unambiguous entries
        info!(
            "Syncing {} unambiguous entries",
            partition.unambiguous_entries.len()
        );
        sync_unambiguous_entries(
            &cfg.destination,
            partition.unambiguous_entries.into_values().collect_vec(),
        )
        .await?;
        // Display the count of remaining ambiguous entries
        info!(
            "There are {} ambiguous entries remaining",
            partition.ambiguous_entries.len()
        );
        // If there are no ambiguous entries, we're done
        if partition.ambiguous_entries.is_empty() {
            info!("No ambiguous entries to process");
            return Ok(());
        }

        // Partition ambiguous entries by CRC32 hash uniqueness
        let partition = UniqueCrc32HashPartitionStrategy::partition(partition.ambiguous_entries);
        info!(
            "Partitioned ambiguous entries by CRC32; {} unambiguous, {} ambiguous",
            partition.unambiguous_entries.len(),
            partition.ambiguous_entries.len()
        );

        // Sync unambiguous entries
        info!(
            "Syncing {} unambiguous entries",
            partition.unambiguous_entries.len()
        );
        sync_unambiguous_entries(
            &cfg.destination,
            partition.unambiguous_entries.into_values().collect_vec(),
        )
        .await?;
        // Display the count of remaining ambiguous entries
        info!(
            "There are {} ambiguous entries remaining",
            partition.ambiguous_entries.len()
        );
        // If there are no ambiguous entries left, we're done
        if partition.ambiguous_entries.is_empty() {
            info!("No ambiguous entries to process");
            return Ok(());
        }

        bail!(
            "There are still {} ambiguous entries remaining. Please resolve them manually.",
            partition.ambiguous_entries.len()
        );
    }
}
