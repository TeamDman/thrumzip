use crate::progress::worker::track_progress;
use crate::zip_entry::ZipEntry;
use color_eyre::eyre::Result;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

pub async fn sync_unambiguous_entries(
    destination: &Path,
    entries: Vec<ZipEntry>,
) -> Result<(), eyre::Error> {
    info!(
        "Beginning sync of {} entries to {}",
        entries.len(),
        destination.display()
    );
    track_progress(
        entries
            .into_iter()
            .map(|item| {
                let destination = item.get_splat_path(&destination, false)?;
                Ok((destination, item))
            })
            .collect::<eyre::Result<Vec<_>>>()?,
        Duration::from_millis(500),
        |progress| info!("Spawning write tasks {progress}"),
        |progress| info!("Waiting for write tasks to complete... {progress}"),
        |progress| info!("Completing write tasks {progress}"),
        |progress| info!("Sync complete! ({progress})"),
        |(destination, item): (PathBuf, ZipEntry)| async move {
            if !destination.exists() {
                item.write_to_file(&destination).await?;
            }
            Ok(item)
        },
        0,
    )
    .await?;

    Ok(())
}
