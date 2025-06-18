use crate::command::GlobalArgs;
use crate::config_state::AppConfig;
use crate::get_zips;
use crate::partition::PartitionStrategy;
use crate::partition::partition_strategy_unique_crc32::UniqueCrc32HashPartitionStrategy;
use crate::partition::partition_strategy_unique_image_hash::UniqueImageHashPartitionStrategy;
use crate::partition::partition_strategy_unique_name::UniqueNamePartitionStrategy;
use crate::read_entries_from_zips;
use crate::size_of_thing::KnownCount;
use crate::size_of_thing::KnownSize;
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
        info!("Loading configuration...");
        let cfg = AppConfig::load()
            .await
            .wrap_err("Failed to load configuration")?;

        info!("Gathering zip files from sources...");
        let (zips, zips_size) = get_zips::get_zips(&cfg).await?;
        info!(
            "Found {} zip files in the source paths ({})",
            zips.len(),
            zips_size.human_size()
        );

        info!("Reading entries from zips...");
        let entries = read_entries_from_zips::read_entries_from_zips(zips).await?;
        info!(
            "Found {} entries ({}) in the source zips",
            entries.len(),
            entries.human_size()
        );

        let mut partition = UniqueNamePartitionStrategy.partition(entries).await?;
        assert!(partition.unprocessed_entries.is_empty());

        sync_unambiguous_entries(
            &cfg.destination,
            partition.unambiguous_entries.into_values().collect_vec(),
        )
        .await?;

        info!(
            "There are {} ambiguous entries remaining ({})",
            partition.ambiguous_entries.count(),
            partition.ambiguous_entries.human_size()
        );
        if partition.ambiguous_entries.is_empty() {
            info!("No ambiguous entries to process");
            return Ok(());
        }

        partition = UniqueCrc32HashPartitionStrategy
            .partition(partition.ambiguous_entries)
            .await?;
        assert!(partition.unprocessed_entries.is_empty());

        sync_unambiguous_entries(
            &cfg.destination,
            partition.unambiguous_entries.into_values().collect_vec(),
        )
        .await?;

        info!(
            "There are {} ambiguous entries remaining",
            partition.ambiguous_entries.count()
        );
        if partition.ambiguous_entries.is_empty() {
            info!("No ambiguous entries to process");
            return Ok(());
        }

        partition = UniqueImageHashPartitionStrategy { stop_after: 10 }
            .partition(partition.ambiguous_entries)
            .await?;
        sync_unambiguous_entries(
            &cfg.destination,
            partition.unambiguous_entries.into_values().collect_vec(),
        )
        .await?;

        info!(
            "There are {} unprocessed entries remaining",
            partition.unprocessed_entries.count()
        );
        info!(
            "There are {} ambiguous entries remaining",
            partition.ambiguous_entries.count()
        );
        if partition.unprocessed_entries.is_empty() && partition.ambiguous_entries.is_empty() {
            info!("No ambiguous entries to process");
            return Ok(());
        }

        bail!(
            "There are still {} unprocessed entries and {} ambiguous entries remaining. Please resolve them manually.",
            partition.unprocessed_entries.count(),
            partition.ambiguous_entries.count()
        );
    }
}
