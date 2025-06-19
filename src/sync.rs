use crate::command::GlobalArgs;
use crate::config_state::AppConfig;
use crate::gather_existing_files::gather_existing_files;
use crate::get_zips;
use crate::path_inside_zip::PathInsideZip;
use crate::progress::worker::track_progress;
use crate::read_entries_from_zips;
use crate::size_of_thing::KnownCount;
use crate::size_of_thing::KnownSize;
use crate::zip_entry::ZipEntry;
use clap::Args;
use color_eyre::eyre::Result;
use color_eyre::eyre::WrapErr;
use eye_config::persistable_state::PersistableState;
use eyre::bail;
use image::load_from_memory;
use img_hash::HashAlg;
use img_hash::HasherConfig;
use itertools::Itertools;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::OsString;
use std::sync::Arc;
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
        let existing_destination_files = Arc::new(existing_destination_files);

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

        let mut not_on_disk: Vec<ZipEntry> = Vec::new();
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
        let entries = not_on_disk;

        // Spawn task to write entries
        let (write_to_disk_tx, mut write_to_disk_rx) =
            tokio::sync::mpsc::unbounded_channel::<(ZipEntry, bool)>();
        let write_to_disk_join_handle = tokio::spawn(async move {
            while let Some((entry, disambiguate)) = write_to_disk_rx.recv().await {
                let destination = entry.get_splat_path(&cfg.destination, disambiguate)?;
                if !destination.exists() {
                    info!("Writing entry to {}", destination.display());
                    entry.write_to_file(&destination).await?;
                }
            }
            eyre::Ok(())
        });

        info!("Partitioning entries by name...");
        let entries_by_name = entries
            .into_iter()
            .into_group_map_by(|entry| entry.path_inside_zip.clone());
        info!(
            "Found {} unique names used by {} entries ({})",
            entries_by_name.len(),
            entries_by_name.count(),
            entries_by_name.human_size()
        );
        let mut ambiguous_entries = HashMap::new();
        for (path_inside_zip, entries) in entries_by_name {
            if entries.len() == 1 {
                let zip_entry = entries.into_iter().next().unwrap();
                if !existing_destination_files.contains_key(&zip_entry.path_inside_zip) {
                    write_to_disk_tx
                        .send((zip_entry, false))
                        .expect("Failed to send entry to writer");
                }
            } else {
                ambiguous_entries.insert(path_inside_zip, entries);
            }
        }
        let entries = ambiguous_entries;
        let hasher_config = Arc::new(HasherConfig::new().hash_alg(HashAlg::Gradient));

        let unprocessed = track_progress(
            entries,
            Duration::from_millis(500),
            |progress| info!("Spawning disambiguation tasks {progress}"),
            |progress| info!("Completing disambiguation tasks {progress}"),
            |_progress, elapsed| info!("Disambiguation complete in {elapsed}!"),
            move |(path_inside_zip, entries): (PathInsideZip, Vec<ZipEntry>)| {
                let write_to_disk_tx2 = write_to_disk_tx.clone();
                let hasher_config = hasher_config.clone();
                async move {
                    // Check CRC32 uniqueness
                    let same_crc = entries
                        .iter()
                        .all(|entry| entry.entry.crc32 == entries[0].entry.crc32);
                    if same_crc {
                        let zip_entry = entries.into_iter().next().unwrap();
                            write_to_disk_tx2
                                .send((zip_entry, false))
                                .expect("Failed to send entry to writer");
                        return Ok(None);
                    }

                    // Check if image hash
                    let image_extensions: HashSet<_> =
                        ["jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp", "heic"]
                            .into_iter()
                            .map(OsString::from)
                            .collect();
                    let is_image = path_inside_zip
                        .extension()
                        .map(|ext| image_extensions.contains(ext))
                        .unwrap_or(false);
                    if is_image {
                    let mut images = Vec::with_capacity(entries.len());
                        let mut failed_to_load_any_images = false;
                        for entry in entries.iter() {
                            let data = entry.bytes().await?.to_vec();
                            let image = load_from_memory(&data);
                            if let Ok(image) = image {
                                images.push(image);
                            } else {
                                failed_to_load_any_images = true;
                                break;
                            }
                        }
                        if !failed_to_load_any_images {
                        let hasher: img_hash::Hasher = hasher_config.to_hasher();
                            let mut hashes = Vec::with_capacity(images.len());
                            for image in images {
                                hashes.push(hasher.hash_image(&image));
                            }
                            let mut ambiguous = false;
                            let mut max_dist = 0;
                            for i in 0..hashes.len() {
                                for j in (i + 1)..hashes.len() {
                                    let d = hashes[i].dist(&hashes[j]);
                                    max_dist = max_dist.max(d);
                                    const THRESHOLD: u32 = 5;
                                    if d > THRESHOLD {
                                        ambiguous = true;
                                        break;
                                    }
                                }
                                if ambiguous {
                                    break;
                                }
                            }
                            info!(
                                "Images {} have hashes {:?}, max_dist={max_dist}, ambiguous={ambiguous}",
                                path_inside_zip.display(),
                                hashes
                                    .iter()
                                    .format_with(", ", |h, f| f(&format_args!("{}", h.to_base64())))
                            );
                            if !ambiguous {
                                // make sure we grab the smallest entry by uncompressed size
                                let zip_entry = entries.into_iter().sorted_by_key(|entry| entry.entry.uncompressed_size).next().unwrap();
                                    write_to_disk_tx2
                                        .send((zip_entry, false))
                                        .expect("Failed to send entry to writer");
                                return Ok(None);
                            }
                        }
                    }

                    Ok(Some((path_inside_zip, entries)))
                }
            },
            24, // we don't want to use all 32 because that probably causes thrashing with cpu scheduling from OS?
        )
        .await?
        .into_iter()
        .flatten()
        .collect_vec();

        info!("Waiting for write tasks to complete...");
        write_to_disk_join_handle.await??;

        if !unprocessed.is_empty() {
            bail!(
                "There are {} entries that could not be processed ({})",
                unprocessed.len(),
                unprocessed.human_size()
            );
        }

        Ok(())
    }
}
