use crate::PathInsideZip;
use crate::partition::Partition;
use crate::partition::PartitionStrategy;
use crate::zip_entry::ZipEntry;
use image::load_from_memory;
use img_hash::HashAlg;
use img_hash::HasherConfig;
use std::collections::HashMap;

/// The entries for which all entries with the same name have the same CRC32 hash are considered unambiguous.
/// Entries with the same name but different CRC32 hashes are considered ambiguous.
///
/// If all entries with the same name have the same CRC32 hash, it is not guaranteed what entry will be selected as the unambiguous entry, with the rest being omitted from the resulting partition.
pub struct UniqueImageHashPartitionStrategy {
    pub stop_after: usize,
}
impl PartitionStrategy for UniqueImageHashPartitionStrategy {
    type Input = HashMap<PathInsideZip, Vec<ZipEntry>>;
    fn label() -> &'static str {
        "image hash uniqueness"
    }

    async fn partition_inner(&self, entries: Self::Input) -> eyre::Result<Partition> {
        let mut rtn = Partition::new_empty();
        let hasher = HasherConfig::new().hash_alg(HashAlg::Gradient).to_hasher();
        for (i, (name, group)) in entries.into_iter().enumerate() {
            if i > self.stop_after {
                rtn.unprocessed_entries.insert(name, group);
                continue;
            }
            let mut images = Vec::new();

            // read the images from the group
            let mut failed_to_load_any_images = false;
            for entry in group.iter() {
                let data = entry.bytes().await?;
                let image = load_from_memory(&data);
                if let Ok(image) = image {
                    images.push(image);
                } else {
                    // the whole group is ambiguous if any entry is not a valid image
                    failed_to_load_any_images = true;
                    break;
                }
            }
            if failed_to_load_any_images {
                assert!(rtn.ambiguous_entries.insert(name, group).is_none());
                continue;
            }

            // hash the images
            let mut hashes = Vec::new();
            for image in images {
                hashes.push(hasher.hash_image(&image));
            }

            // if all hashes are the same, we can consider the group unambiguous
            if hashes.iter().all(|h| *h == hashes[0]) {
                rtn.unambiguous_entries
                    .insert(name, group.into_iter().next().unwrap());
            } else {
                rtn.ambiguous_entries.insert(name, group);
            }
        }
        Ok(rtn)
    }
}
