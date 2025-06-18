use crate::progress::Progress;
use crate::size_of_thing::KnownSize;
use crate::zip_entry::ZipEntry;
use color_eyre::eyre::Result;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use std::usize;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tracing::info;
use uom::si::f64::Information;
use uom::si::information::byte;

pub async fn sync_unambiguous_entries(
    destination: &Path,
    entries: Vec<ZipEntry>,
) -> Result<(), eyre::Error> {
    info!(
        "Beginning sync of {} entries to {}",
        entries.len(),
        destination.display()
    );

    let mut progress = {
        Progress::new(
            entries.len(),
            Information::new::<byte>(
                entries.iter().map(|e| e.size_in_bytes()).sum::<usize>() as f64
            ),
        )
    };
    // let progress_interval = Duration::ZERO;
    let progress_interval = Duration::from_millis(500);
    let mut last_progress = Instant::now();

    // Spawn work
    let mut join_set: JoinSet<eyre::Result<Response>> = JoinSet::new();
    // let throughput: Option<Arc<Semaphore>> = Arc::new(Semaphore::new(10));
    let throughput: Option<Arc<Semaphore>> = None;
    struct Response {
        skipped: usize,
        written: usize,
        size: Information,
    }
    for entry in entries {
        let throughput = throughput.clone();
        let processed_bytes = entry.size_of();
        let destination = destination.to_path_buf();
        join_set.spawn(async move {
            let _permit = match throughput.as_ref() {
                Some(sem) => Some(sem.acquire().await),
                None => None,
            };
            let dest = entry.get_splat_path(&destination, false)?;
            let (skipped, written) = if dest.exists() {
                (1, 0)
            } else {
                entry.write_to_file(&dest).await?;
                (0, 1)
            };
            Ok(Response {
                skipped,
                written,
                size: Information::new::<byte>(entry.size_in_bytes() as f64),
            })
        });

        // Track and log progress
        progress.track(1, 0, processed_bytes);
        if Instant::now().duration_since(last_progress) >= progress_interval {
            let x1 = Instant::now();
            info!("Spawning write tasks {progress}");
            let elapsed = x1.elapsed();
            info!(
                "Took {} to display progress",
                humantime::format_duration(elapsed)
            );
            last_progress = Instant::now();
        }
    }

    // Complete work
    progress.reset();
    info!("Waiting for write tasks to complete... {progress}");
    while let Some(res) = join_set.join_next().await {
        let res = res??;
        progress.track(res.written + res.skipped, res.skipped, res.size);
        if Instant::now().duration_since(last_progress) >= progress_interval {
            info!("Completing write tasks {progress}");
            last_progress = Instant::now();
        }
    }
    info!("Sync complete! ({progress})");
    Ok(())
}
