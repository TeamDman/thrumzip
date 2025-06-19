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
                if let Some(parent_dir_named_zip) = existing_file_path.parent().filter(|parent| {
                    parent
                        .file_name()
                        .is_some_and(|name| name.to_string_lossy().ends_with(".zip"))
                }) {
                    let path_inside_zip = {
                        PathInsideZip::from(Arc::new(
                            parent_dir_named_zip
                                .parent()
                                .unwrap()
                                .join(existing_file_path.file_name().unwrap())
                                .strip_prefix(dir)
                                .unwrap()
                                .to_path_buf(),
                        ))
                    };
                    files.push(ExistingFile::Ambiguous {
                        path_inside_zip,
                        zip_name: parent_dir_named_zip
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                        path_on_disk: existing_file_path.clone(),
                        size,
                    });
                } else {
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
    }
    Ok(files)
}

#[cfg(test)]
mod test {
    use crate::gather_existing_files::gather_existing_files;
    use crate::get_zips;
    use crate::init_tracing;
    use crate::path_inside_zip::PathInsideZip;
    use crate::read_entries_from_zips;
    use crate::size_of_thing::KnownSize;
    use crate::state::profiles::Profile;
    use crate::state::profiles::Profiles;
    use eyre::Context;
    use eyre::OptionExt;
    use itertools::Itertools;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tracing::Level;
    use tracing::info;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        let profile = Profile::new_example();
        let files = gather_existing_files(&profile.destination).await?;
        println!("{:#?}", files);
        assert!(files.iter().any(|f| f.is_ambiguous()));
        Ok(())
    }
    #[tokio::test]
    #[ignore]
    async fn it_works2() -> eyre::Result<()> {
        let profile = Profiles::load_and_get_active_profile().await?;
        let files = gather_existing_files(&profile.destination).await?;
        for file in &files {
            if file.path_inside_zip().to_str().unwrap().contains("feed") {
                println!("{:#?}", file);
            }
        }
        assert!(files.iter().any(|f| f.is_ambiguous()));
        Ok(())
    }
    #[tokio::test]
    #[ignore]
    async fn it_works3() -> eyre::Result<()> {
        color_eyre::install().wrap_err("Failed to install color_eyre")?;
        init_tracing::init_tracing(Level::INFO);
        let profile = Profiles::load_and_get_active_profile().await?;

        let existing_destination_files = gather_existing_files(&profile.destination)
            .await?
            .into_iter()
            .into_group_map_by(|entry| entry.path_inside_zip().to_owned());
        let (zips, zips_size) = get_zips::get_zips(&profile.sources).await?;
        info!(
            "Found {} zip files in the source paths ({})",
            zips.len(),
            zips_size.human_size()
        );

        info!("Reading entries from zips...");
        let entries = read_entries_from_zips::read_entries_from_zips(zips).await?;
        info!(
            "Found {} entries ({}) in the source zips",
            entries.len(),
            entries.human_size()
        );

        println!(
            "Existing destination files: {:#?}",
            existing_destination_files
                .iter()
                .filter(|x| x.0.to_str().unwrap().contains("feed"))
                .collect::<Vec<_>>()
        );
        println!(
            "Entries: {:#?}",
            entries
                .iter()
                .filter(|e| e.path_inside_zip.to_str().unwrap().contains("feed"))
                .collect::<Vec<_>>()
        );

        let feed_from_dest = existing_destination_files
            .get(&PathInsideZip::from(Arc::new(
                "preferences/feed/feed.json".into(),
            )))
            .ok_or_eyre("No feed.json found in destination directory")?;
        let feed_from_zip = entries
            .iter()
            .find(|e| {
                e.path_inside_zip
                    == PathInsideZip::from(Arc::new("preferences/feed/feed.json".into()))
            })
            .ok_or_eyre("No feed.json found in source zips")?;
        assert!(
            feed_from_dest.len() > 0,
            "Expected at least one feed.json in destination directory"
        );
        assert_eq!(
            **feed_from_zip.path_inside_zip,
            PathBuf::from("preferences/feed/feed.json"),
            "Expected feed.json entry in source zips"
        );
        Ok(())
    }
}
