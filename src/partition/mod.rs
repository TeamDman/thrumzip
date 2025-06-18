pub mod partition_strategy_unique_crc32;
pub mod partition_strategy_unique_image_hash;
pub mod partition_strategy_unique_name;

use crate::PathInsideZip;
use crate::size_of_thing::KnownCount;
use crate::size_of_thing::KnownSize;
use crate::zip_entry::ZipEntry;
use std::collections::HashMap;
use tracing::info;

/// Created using a partition strategy's partition method.
#[derive(Default)]
pub struct Partition {
    /// Entries with names for which no other entries with the same name and different CRC exist.
    pub unambiguous_entries: HashMap<PathInsideZip, ZipEntry>,
    /// Entries with names that are ambiguous (i.e., multiple entries with the same name and different CRC exist).
    pub ambiguous_entries: HashMap<PathInsideZip, Vec<ZipEntry>>,
    /// Entries that were not processed by the partition strategy.
    pub unprocessed_entries: HashMap<PathInsideZip, Vec<ZipEntry>>,
}
impl Partition {
    pub fn new_empty() -> Self {
        Self::default()
    }

    pub fn new_unprocessed(entries: HashMap<PathInsideZip, Vec<ZipEntry>>) -> Self {
        Self {
            unambiguous_entries: HashMap::new(),
            ambiguous_entries: HashMap::new(),
            unprocessed_entries: entries,
        }
    }
}

pub trait PartitionStrategy {
    type Input: KnownCount + KnownSize;
    fn label() -> &'static str;
    async fn partition(&self, entries: Self::Input) -> eyre::Result<Partition> {
        let start = std::time::Instant::now();
        let len = entries.count();
        let size = entries.human_size();
        info!(
            "Partitioning {} entries ({}) by {}",
            len,
            size,
            Self::label()
        );
        let rtn = self.partition_inner(entries).await?;
        info!(
            "Partitioned {} entries in {} --> {} unambiguous, {} ambiguous by {}",
            len,
            humantime::format_duration(start.elapsed()),
            rtn.unambiguous_entries.len(),
            rtn.ambiguous_entries.len(),
            Self::label()
        );
        Ok(rtn)
    }
    async fn partition_inner(&self, entries: Self::Input) -> eyre::Result<Partition>;
}
