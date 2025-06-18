use crate::progress::Progress;
use crate::zip_entry::ZipEntry;
use color_eyre::eyre::Result;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;
use tokio::task::JoinSet;
use tracing::info;
use uom::si::f64::Information;
use uom::si::information::byte;

pub async fn sync_unambiguous_entries(
    destination: &Path,
    entries: Vec<ZipEntry>,
) -> Result<(), eyre::Error> {
    let mut progress = Progress::new(
        entries.len(),
        Information::new::<byte>(
            entries
                .iter()
                .map(|e| e.entry.uncompressed_size)
                .sum::<u64>() as f64,
        ),
    );
    let mut last_progress = Instant::now();
    let mut skipped_entries = 0;
    let mut written_entries = 0;
    info!(
        "Beginning sync of {} unambiguous entries to {}",
        entries.len(),
        destination.display()
    );
    let progress_interval = Duration::from_millis(500);
    let mut join_set: JoinSet<eyre::Result<Response>> = JoinSet::new();
    struct Response {
        skipped: usize,
        written: usize,
        size: Information,
        dest: PathBuf,
    }
    for entry in entries {
        let dest = entry.get_splat_path(&destination, false)?;
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
