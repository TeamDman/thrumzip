use crate::PathToZip;
use crate::config_state::AppConfig;
use std::path::PathBuf;
use std::sync::Arc;

pub async fn get_zips(cfg: &AppConfig) -> Result<Vec<PathToZip>, eyre::Error> {
    let mut zips = Vec::new();
    for src in &cfg.sources {
        let dir = PathBuf::from(src);
        if !dir.is_dir() {
            continue;
        }
        let mut rd = tokio::fs::read_dir(&dir).await?;
        while let Some(e) = rd.next_entry().await? {
            let path = e.path();
            if path
                .extension()
                .and_then(|s| s.to_str())
                .map_or(false, |ext| ext.eq_ignore_ascii_case("zip"))
            {
                zips.push(PathToZip {
                    inner: Arc::new(path),
                });
            }
        }
    }
    Ok(zips)
}
