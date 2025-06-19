use crate::existing_file::ExistingFile;
use crate::path_inside_zip::PathInsideZip;
use std::path::Path;
use std::sync::Arc;
use uom::si::f64::Information;
use uom::si::information::byte;

pub async fn gather_existing_files(dir: &Path) -> eyre::Result<Vec<ExistingFile>> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        if !d.exists() {
            continue; // Skip non-existent directories
        }
        let mut rd = tokio::fs::read_dir(&d).await?;
        while let Some(e) = rd.next_entry().await? {
            let p = e.path();
            let m = tokio::fs::metadata(&p).await?;
            let size = Information::new::<byte>(m.len() as f64);
            if m.is_dir() {
                stack.push(p);
            } else {
                // Determine if parent dir ends with .zip
                if let Some(parent) = p.parent() {
                    if let Some(parent_name) = parent.file_name().and_then(|n| n.to_str()) {
                        if parent_name.ends_with(".zip") {
                            // Omit the .zip parent dir from path_inside_zip
                            let rel_path = p.strip_prefix(parent).unwrap();
                            files.push(ExistingFile::Ambiguous {
                                path_inside_zip: PathInsideZip::from(Arc::new(
                                    rel_path.to_path_buf(),
                                )),
                                zip_name: parent_name.to_string(),
                                paths_on_disk: vec![p.clone()],
                                size,
                            });
                            continue;
                        }
                    }
                }
                files.push(ExistingFile::Unambiguous {
                    path_inside_zip: PathInsideZip::from(Arc::new(
                        p.strip_prefix(dir).unwrap().to_path_buf(),
                    )),
                    path_on_disk: p.clone(),
                    size,
                });
            }
        }
    }
    Ok(files)
}
