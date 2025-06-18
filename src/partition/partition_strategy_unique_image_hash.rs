use crate::PathInsideZip;
use crate::partition::Partition;
use crate::partition::PartitionStrategy;
use crate::progress::worker::track_progress;
use crate::zip_entry::ZipEntry;
use image::load_from_memory;
use img_hash::HashAlg;
use img_hash::HasherConfig;
use itertools::Itertools;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tracing::info;

/// The entries for which all entries with the same name have the same CRC32 hash are considered unambiguous.
/// Entries with the same name but different CRC32 hashes are considered ambiguous.
///
/// If all entries with the same name have the same CRC32 hash, it is not guaranteed what entry will be selected as the unambiguous entry, with the rest being omitted from the resulting partition.
pub struct UniqueImageHashPartitionStrategy {
    pub stop_after: (usize, Duration),
}
impl PartitionStrategy for UniqueImageHashPartitionStrategy {
    type Input = HashMap<PathInsideZip, Vec<ZipEntry>>;
    fn label() -> &'static str {
        "image hash uniqueness"
    }

    async fn partition_inner(&self, entries: Self::Input) -> eyre::Result<Partition> {
        let mut rtn = Partition::new_empty();
        let hasher_config = Arc::new(HasherConfig::new().hash_alg(HashAlg::Gradient));
        let start = Instant::now();
        let stop_after_count = self.stop_after.0;
        let stop_after_instant = start + self.stop_after.1;
        track_progress(
            entries
                .into_iter()
                .enumerate()
                .map(|(i, (name, group))| (i, name, group))
                .collect_vec(),
            Duration::from_millis(500),
            |progress| info!("Partitioning image groups {progress}"),
            |progress| info!("Processing image group {progress}"),
            |_progress, elapsed| info!("Partitioning complete in {elapsed} !"),
            {
                let hasher_config = Arc::clone(&hasher_config);
                move |(i, name, group)| {
                    let hasher_config = Arc::clone(&hasher_config);
                    async move {
                        let mut images = Vec::with_capacity(group.len());
                        let mut failed_to_load_any_images = false;
                        for entry in group.iter() {
                            if i > stop_after_count || Instant::now() >= stop_after_instant {
                                return Ok((name, group, None));
                            }

                            let data = entry.bytes().await?.to_vec();
                            let image = load_from_memory(&data);
                            if let Ok(image) = image {
                                images.push(image);
                            } else {
                                failed_to_load_any_images = true;
                                break;
                            }
                        }
                        if failed_to_load_any_images {
                            return Ok((name, group, Some(false)));
                        }
                        let hasher = hasher_config.to_hasher();
                        let mut hashes = Vec::with_capacity(images.len());
                        for image in images {
                            if i > stop_after_count || Instant::now() >= stop_after_instant {
                                return Ok((name, group, None));
                            }
                            hashes.push(hasher.hash_image(&image));
                        }
                        let all_same = hashes.iter().all(|h| *h == hashes[0]);
                        Ok((name, group, Some(all_same)))
                    }
                }
            },
            0,
        )
        .await?
        .into_iter()
        .for_each(|(name, group, result)| match result {
            None => {
                rtn.unprocessed_entries.insert(name, group);
            }
            Some(false) => {
                assert!(rtn.ambiguous_entries.insert(name, group).is_none());
            }
            Some(true) => {
                assert!(
                    rtn.unambiguous_entries
                        .insert(name, group.into_iter().next().unwrap())
                        .is_none()
                );
            }
        });
        Ok(rtn)
    }
}
