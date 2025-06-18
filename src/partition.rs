use crate::PathInsideZip;
use crate::zip_entry::ZipEntry;
use std::collections::HashMap;

/// Created using a partition strategy's partition method.
#[derive(Default)]
pub struct Partition {
    /// Entries with names for which no other entries with the same name and different CRC exist.
    pub unambiguous_entries: HashMap<PathInsideZip, ZipEntry>,
    /// Entries with names that are ambiguous (i.e., multiple entries with the same name and different CRC exist).
    pub ambiguous_entries: HashMap<PathInsideZip, Vec<ZipEntry>>,
}
impl Partition {
    pub fn len(&self) -> usize {
        self.unambiguous_entries.len() + self.ambiguous_entries.len()
    }
    pub fn new_empty() -> Self {
        Self::default()
    }
}

pub trait PartitionStrategy {
    type Input;
    fn partition(entries: Self::Input) -> Partition;
}
