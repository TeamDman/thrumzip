use crate::PathInsideZip;
use crate::PathToZip;
use crate::get_splat_path::DisambiguationStrategy;
use crate::get_splat_path::get_splat_path;
use eyre::Context;
use positioned_io::RandomAccessFile;
use rc_zip::parse::Entry;
use rc_zip::parse::EntryKind;
use rc_zip_tokio::HasCursor;
use rc_zip_tokio::entry_reader::EntryReader;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;

#[derive(Clone)]
pub struct ZipEntry {
    pub path_to_zip: PathToZip,
    pub path_inside_zip: PathInsideZip,
    pub file: Arc<RandomAccessFile>,
    pub entry: Entry,
}
impl ZipEntry {
    pub fn reader(&self) -> impl AsyncRead + Send + 'static {
        EntryReader::new(&self.entry, |offset| self.file.cursor_at(offset))
    }
    pub fn is_file(&self) -> bool {
        self.entry.kind() == EntryKind::File
    }
    /// Reads the entire entry into a vector.
    pub async fn bytes(&self) -> tokio::io::Result<Vec<u8>> {
        let mut v = Vec::new();
        self.reader().read_to_end(&mut v).await?;
        Ok(v)
    }
    pub fn get_splat_path(&self, dest_dir: &Path, disambiguate: bool) -> eyre::Result<PathBuf> {
        Ok(get_splat_path(
            &self.path_inside_zip,
            dest_dir,
            disambiguate
                .then(|| DisambiguationStrategy::Some {
                    path_to_zip: self.path_to_zip.clone(),
                })
                .unwrap_or_else(|| DisambiguationStrategy::None),
        )?)
    }
    pub async fn write_to_file(&self, dest: &Path) -> eyre::Result<()> {
        let Some(parent) = dest.parent() else {
            return Err(eyre::eyre!(
                "Destination path {} has no parent directory.",
                dest.display()
            ));
        };
        _ = tokio::fs::create_dir_all(parent).await;
        let data = self.bytes().await?;
        tokio::fs::write(&dest, data)
            .await
            .wrap_err_with(|| eyre::eyre!("Failed to write to {}", dest.display()))?;
        Ok(())
    }
}
