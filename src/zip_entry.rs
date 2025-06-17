use crate::PathInsideZip;
use crate::PathToZip;
use eyre::OptionExt;
use positioned_io::RandomAccessFile;
use rc_zip::parse::Entry;
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
    /// Reads the entire entry into a vector.
    pub async fn bytes(&self) -> tokio::io::Result<Vec<u8>> {
        let mut v = Vec::new();
        self.reader().read_to_end(&mut v).await?;
        Ok(v)
    }
    pub async fn splat_into_dir(&self, dest_dir: &Path, disambiguate: bool) -> eyre::Result<PathBuf> {
        let splatted = if disambiguate {
            let parent = self.path_inside_zip.parent().unwrap_or_else(|| std::path::Path::new(""));
            let zip_file_name = <PathToZip as AsRef<std::path::Path>>::as_ref(&self.path_to_zip)
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("unknown_zip"));
            let file_name = <PathInsideZip as AsRef<std::path::Path>>::as_ref(&self.path_inside_zip)
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("unknown_file"));
            parent.join(zip_file_name).join(file_name)
        } else {
            <PathInsideZip as AsRef<std::path::Path>>::as_ref(&self.path_inside_zip).to_path_buf()
        };
        let dest = dest_dir.join(splatted);
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let data = self.bytes().await?;
        tokio::fs::write(&dest, data).await?;
        Ok(dest)
    }
}
