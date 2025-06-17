use std::path::{Path, PathBuf};

pub async fn try_count_files(dir: &Path) -> eyre::Result<usize> {
    let mut count = 0;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let mut rd = tokio::fs::read_dir(&d).await?;
        while let Some(e) = rd.next_entry().await? {
            let p = e.path();
            let m = tokio::fs::metadata(&p).await?;
            if m.is_dir() {
                stack.push(p);
            } else {
                count += 1;
            }
        }
    }
    Ok(count)
}

pub async fn count_files(dest_dir: &PathBuf) -> usize {
    let dest_count = if tokio::fs::try_exists(dest_dir).await.unwrap_or(false) {
        try_count_files(dest_dir).await.unwrap_or(0)
    } else {
        0
    };
    dest_count
}