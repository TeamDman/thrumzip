use crate::partition::Partition;
use crate::partition::PartitionStrategy;
use crate::zip_entry::ZipEntry;
use itertools::Itertools;

pub struct UniqueNamePartitionStrategy;
impl PartitionStrategy for UniqueNamePartitionStrategy {
    type Input = Vec<ZipEntry>;
    
    fn label() -> &'static str {
        "name uniqueness"
    }
    
    async fn partition_inner(&self, entries: Self::Input) -> eyre::Result<Partition> {
        let entries_by_name = entries
            .into_iter()
            .into_group_map_by(|entry| entry.path_inside_zip.clone());
        let mut rtn = Partition::new_empty();
        for (path, group) in entries_by_name {
            if group.len() == 1 {
                rtn.unambiguous_entries
                    .insert(path, group.into_iter().next().unwrap());
            } else {
                rtn.ambiguous_entries.insert(path, group);
            }
        }
        Ok(rtn)
    }
}
