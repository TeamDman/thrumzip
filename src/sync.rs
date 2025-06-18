use crate::command::GlobalArgs;
use crate::config_state::AppConfig;
use crate::count_files::count_files;
use crate::get_zips;
use crate::partition::Partition;
use crate::progress::Progress;
use crate::read_entries_from_zips;
use clap::Args;
use color_eyre::eyre::Result;
use color_eyre::eyre::WrapErr;
use eye_config::persistable_state::PersistableState;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;
use tokio::task::JoinSet;
use tracing::info;
use uom::si::f64::Information;
use uom::si::information::byte;

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

        // Scan entries in parallel and build entry handles
        let entries = read_entries_from_zips::read_entries_from_zips(zips).await?;
        info!("Found {} entries in the source zips", entries.len());
        let entries = Partition::from(entries);
        info!(
            "Partitioned entries; {} unambiguous, {} ambiguous",
            entries.unambiguous_entries.len(),
            entries.ambiguous_entries.len()
        );

        let mut progress = Progress::new(
            entries.unambiguous_entries.len(),
            Information::new::<byte>(
                entries
                    .unambiguous_entries
                    .values()
                    .map(|e| e.entry.uncompressed_size)
                    .sum::<u64>() as f64,
            ),
        );
        let mut last_progress = Instant::now();
        let mut skipped_entries = 0;
        let mut written_entries = 0;

        info!(
            "Beginning sync of {} entries to {}",
            entries.unambiguous_entries.len(),
            cfg.destination.display()
        );
        let progress_interval = Duration::from_millis(500);
        let mut join_set: JoinSet<eyre::Result<Response>> = JoinSet::new();
        struct Response {
            skipped: usize,
            written: usize,
            size: Information,
            dest: PathBuf,
        }
        for entry in entries.unambiguous_entries.into_values() {
            let dest = entry.get_splat_path(&cfg.destination, false)?;
            join_set.spawn(async move {
                let (skipped, written) = if dest.exists() {
                    (1, 0)
                } else {
                    entry.write_to_file(&dest).await?;
                    (0, 1)
                };
                Ok(Response {
                    skipped,
                    written,
                    size: Information::new::<byte>(entry.entry.uncompressed_size as f64),
                    dest,
                })
            });
        }
        while let Some(res) = join_set.join_next().await {
            let res = res??;
            skipped_entries += res.skipped;
            written_entries += res.written;
            progress.track(1, res.size);
            if Instant::now().duration_since(last_progress) >= progress_interval {
                info!("Synced: {}", res.dest.display());
                info!("Skipped: {skipped_entries} Written: {written_entries} ({progress})");
                last_progress = Instant::now();
            }
        }
        info!("Sync complete! Skipped={skipped_entries} Written={written_entries} ({progress})");

        Ok(())
    }
}
