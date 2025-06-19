use crate::path_to_zip::PathToZip;
use std::path::Path;
use std::sync::Arc;
use uom::si::f64::Information;
use uom::si::information::byte;

pub async fn get_zips<T: AsRef<Path>>(
    sources: impl IntoIterator<Item = T>,
) -> Result<(Vec<PathToZip>, Information), eyre::Error> {
    let mut zips = Vec::new();
    let mut total_size: Information = Information::new::<byte>(0.0);
    for src in sources {
        let dir = src.as_ref().to_path_buf();
        if !dir.is_dir() {
            continue;
        }
        let mut rd = tokio::fs::read_dir(&dir).await?;
        while let Some(e) = rd.next_entry().await? {
            let path = e.path();
            if path
                .extension()
                .and_then(|s| s.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
            {
                let meta = tokio::fs::metadata(&path).await?;
                total_size += Information::new::<byte>(meta.len() as f64);
                zips.push(PathToZip::new(Arc::new(path)));
            }
        }
    }
    Ok((zips, total_size))
}
