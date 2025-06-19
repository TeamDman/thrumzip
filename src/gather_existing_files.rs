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
        while let Some(existing_file_dir_entry) = rd.next_entry().await? {
            let existing_file_path = existing_file_dir_entry.path();
            let metadata = tokio::fs::metadata(&existing_file_path).await?;
            let size = Information::new::<byte>(metadata.len() as f64);
            if metadata.is_dir() {
                stack.push(existing_file_path);
            } else {
                // Determine if parent dir ends with .zip
                let parent = existing_file_path.parent().unwrap_or(Path::new("."));
                if let Some(parent_name) = parent.file_name().and_then(|n| n.to_str()) {
                    if parent_name.ends_with(".zip") {
                        // Omit the .zip parent dir from path_inside_zip
                        let rel_path = existing_file_path.strip_prefix(parent).unwrap();
                        files.push(ExistingFile::Ambiguous {
                            path_inside_zip: PathInsideZip::from(Arc::new(rel_path.to_path_buf())),
                            zip_name: parent_name.to_string(),
                            paths_on_disk: vec![existing_file_path.clone()],
                            size,
                        });
                        continue;
                    }
                }
                files.push(ExistingFile::Unambiguous {
                    path_inside_zip: PathInsideZip::from(Arc::new(
                        existing_file_path.strip_prefix(dir).unwrap().to_path_buf(),
                    )),
                    path_on_disk: existing_file_path.clone(),
                    size,
                });
            }
        }
    }
    Ok(files)
}

#[cfg(test)]
mod test {
    use crate::gather_existing_files::gather_existing_files;
    use crate::state::profiles::Profile;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        let profile = Profile::new_example();
        let files = gather_existing_files(&&profile.destination).await?;
        println!("{:#?}", files);
        assert!(files.iter().any(|f| f.is_ambiguous()));
        Ok(())
    }
}
