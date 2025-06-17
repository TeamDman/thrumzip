use crate::PathInsideZip;
use crate::zip_entry::ZipEntry;
use itertools::Itertools;
use std::collections::HashMap;

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
}

impl From<Vec<ZipEntry>> for Partition {
    fn from(entries: Vec<ZipEntry>) -> Self {
        let entries_by_name = entries
            .into_iter()
            .into_group_map_by(|entry| entry.path_inside_zip.clone());
        let mut rtn = Self::default();
        for (path, group) in entries_by_name {
            if group.len() == 1 {
                rtn.unambiguous_entries.insert(path, group.into_iter().next().unwrap());
            } else {
                rtn.ambiguous_entries.insert(path, group);
            }
        }
        rtn
    }
}
