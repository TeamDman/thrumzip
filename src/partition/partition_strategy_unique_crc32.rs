use crate::PathInsideZip;
use crate::partition::Partition;
use crate::partition::PartitionStrategy;
use crate::zip_entry::ZipEntry;
use std::collections::HashMap;

/// The entries for which all entries with the same name have the same CRC32 hash are considered unambiguous.
/// Entries with the same name but different CRC32 hashes are considered ambiguous.
///
/// If all entries with the same name have the same CRC32 hash, it is not guaranteed what entry will be selected as the unambiguous entry, with the rest being omitted from the resulting partition.
pub struct UniqueCrc32HashPartitionStrategy;
impl PartitionStrategy for UniqueCrc32HashPartitionStrategy {
    
    type Input = HashMap<PathInsideZip, Vec<ZipEntry>>;
    fn label() -> &'static str {
        "CRC32 uniqueness"
    }

    async fn partition_inner(&self, entries: Self::Input) -> eyre::Result<Partition> {
        let mut rtn = Partition::new_empty();
        for (name, group) in entries {
            let same_crc = group
                .iter()
                .all(|entry| entry.entry.crc32 == group[0].entry.crc32);
            if same_crc {
                rtn.unambiguous_entries
                    .insert(name, group.into_iter().next().unwrap());
            } else {
                rtn.ambiguous_entries.insert(name, group);
            }
        }
        Ok(rtn)
    }
}
