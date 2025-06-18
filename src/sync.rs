use crate::command::GlobalArgs;
use crate::config_state::AppConfig;
use crate::gather_existing_files::gather_existing_files;
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
use std::collections::HashMap;
use std::ops::Not;
use std::time::Duration;
use tracing::info;

#[derive(Args)]
pub struct SyncCommand;

impl SyncCommand {
    pub async fn handle(self, _global: GlobalArgs) -> Result<()> {
        info!("Loading configuration...");
        let cfg = AppConfig::load()
            .await
            .wrap_err("Failed to load configuration")?;

        info!(
            "Gathering files from destination: {}",
            cfg.destination.display()
        );
        let existing_destination_files = gather_existing_files(&cfg.destination)
            .await?
            .into_iter()
            .into_group_map_by(|entry| entry.path_inside_zip().to_owned());
        info!(
            "Found {} files in the destination ({})",
            existing_destination_files.len(),
            existing_destination_files.human_size()
        );

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

        let mut not_on_disk = Vec::new();
        for entry in entries {
            if !existing_destination_files.contains_key(&entry.path_inside_zip) {
                not_on_disk.push(entry);
            }
        }
        info!(
            "There are {} entries not on disk ({})",
            not_on_disk.len(),
            not_on_disk.human_size()
        );

        let mut partition = UniqueNamePartitionStrategy.partition(not_on_disk).await?;
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

        let partition_strategy = UniqueImageHashPartitionStrategy {
            // stop_after: (std::usize::MAX, Duration::from_secs(10)),
            stop_after: (std::usize::MAX, Duration::MAX),
            // stop_after: (3, Duration::MAX),
            similarity_threshold: 5,
        };
        partition = partition_strategy
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

        if partition.unprocessed_entries.is_empty().not() {
            let mut ambiguous = HashMap::new();
            loop {
                let new_partition = partition_strategy
                    .partition(partition.unprocessed_entries)
                    .await?;

                sync_unambiguous_entries(
                    &cfg.destination,
                    new_partition
                        .unambiguous_entries
                        .into_values()
                        .collect_vec(),
                )
                .await?;

                ambiguous.extend(new_partition.ambiguous_entries);
                partition.unprocessed_entries = new_partition.unprocessed_entries;
                if partition.unprocessed_entries.is_empty() {
                    break;
                }
            }
            partition.ambiguous_entries = ambiguous;
        }

        bail!(
            "There are still {} unprocessed entries and {} ambiguous entries remaining. Please resolve them manually.",
            partition.unprocessed_entries.count(),
            partition.ambiguous_entries.count()
        );
    }
}
