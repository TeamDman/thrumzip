use crate::zip_entry::ZipEntry;
use crate::PathInsideZip;
use crate::PathToZip;
use eyre::bail;
use positioned_io::RandomAccessFile;
use rc_zip_tokio::ReadZip;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task::JoinSet;
use tracing::info;
use tracing::warn;

pub async fn read_entries_from_zips(zips: Vec<PathToZip>) -> eyre::Result<Vec<ZipEntry>> {
    info!("Reading entries from {} zips", zips.len());
    if zips.is_empty() {
        warn!("No zips provided, returning empty defaults.");
        return Ok(Default::default());
    }

    let mut tasks: JoinSet<Result<Vec<ZipEntry>, eyre::Error>> = JoinSet::new();
    for path_to_zip in zips {
        tasks.spawn(get_entries_from_zip(path_to_zip));
    }

    let mut rtn = Vec::with_capacity(tasks.len());
    while let Some(res) = tasks.join_next().await {
        let zip_entries = res??;
        rtn.extend(zip_entries);
    }
    Ok(rtn)
}

async fn get_entries_from_zip(path_to_zip: PathToZip) -> eyre::Result<Vec<ZipEntry>> {
    let file = Arc::new(RandomAccessFile::open(path_to_zip.clone())?);
    let archive = file.read_zip().await?;
    let entries = archive.into_entries();
    let mut rtn = Vec::with_capacity(entries.len());
    for entry in entries {
        let Some(path_inside_zip) = entry.sanitized_name() else {
            bail!(
                "Entry {:?} in zip {} has no sanitized name, cannot process it.",
                entry.name,
                path_to_zip.display()
            );
        };
        let zip_entry = ZipEntry {
            path_to_zip: path_to_zip.clone(),
            path_inside_zip: PathInsideZip::new(PathBuf::from(path_inside_zip)),
            file: file.clone(),
            entry,
        };
        rtn.push(zip_entry);
    }

    Ok(rtn)
}
