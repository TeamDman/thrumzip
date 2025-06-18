use crate::PathInsideZip;
use crate::partition::Partition;
use crate::partition::PartitionResponse;
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
    /// Hamming distance threshold for image similarity.
    pub similarity_threshold: u32,
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
        let stop_after_duration = self.stop_after.1;
        let similarity_threshold = self.similarity_threshold;
        track_progress(
            entries
                .into_iter()
                .enumerate()
                .map(|(i, (name, group))| (i, name, group))
                .collect_vec(),
            Duration::from_millis(500),
            |progress| info!("Partitioning image groups {progress}"),
            |progress| info!("Processed image group {progress}"),
            |_progress, elapsed| info!("Partitioning complete in {elapsed} !"),
            {
                let hasher_config = Arc::clone(&hasher_config);
                move |(i, name, group)| {
                    let hasher_config = Arc::clone(&hasher_config);

                    async move {
                        if i > stop_after_count || start.elapsed() >= stop_after_duration {
                            return Ok(PartitionResponse::Unprocessed(name, group));
                        }
                        let mut images = Vec::with_capacity(group.len());
                        let mut failed_to_load_any_images = false;
                        for entry in group.iter() {
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
                            return Ok(PartitionResponse::Ambiguous(name, group));
                        }
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
                                if d > similarity_threshold {
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
                            name.display(),
                            hashes
                                .iter()
                                .format_with(", ", |h, f| f(&format_args!("{}", h.to_base64())))
                        );
                        Ok(if ambiguous {
                            PartitionResponse::Ambiguous(name, group)
                        } else {
                            PartitionResponse::Unambiguous(name, group.into_iter().next().unwrap())
                        })
                    }
                }
            },
            50,
        )
        .await?
        .into_iter()
        .for_each(|response| match response {
            PartitionResponse::Unprocessed(name, group) => {
                rtn.unprocessed_entries.insert(name, group);
            }
            PartitionResponse::Ambiguous(name, group) => {
                assert!(rtn.ambiguous_entries.insert(name, group).is_none());
            }
            PartitionResponse::Unambiguous(name, group) => {
                assert!(rtn.unambiguous_entries.insert(name, group).is_none());
            }
        });
        Ok(rtn)
    }
}
