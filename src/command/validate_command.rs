use crate::command::GlobalArgs;
use crate::existing_file::ExistingFile;
use crate::gather_existing_files::gather_existing_files;
use crate::get_zips;
use crate::path_inside_zip::PathInsideZip;
use crate::progress::worker::track_progress;
use crate::read_entries_from_zips;
use crate::size_of_thing::KnownCount;
use crate::size_of_thing::KnownSize;
use crate::state::profiles::Profiles;
use crate::zip_entry::ZipEntry;
use clap::Args;
use color_eyre::eyre::Result;
use color_eyre::eyre::WrapErr;
use image::load_from_memory;
use img_hash::HashAlg;
use img_hash::HasherConfig;
use itertools::Itertools;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::OsString;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;
use tracing::warn;

#[derive(Args)]
pub struct ValidateCommand;

impl ValidateCommand {
    pub async fn handle(self, _global: GlobalArgs) -> Result<()> {
        info!("Loading profile...");
        let app_profile = Profiles::load_and_get_active_profile()
            .await
            .wrap_err("Failed to load active profile")?;

        info!(
            "Gathering files from destination: {}",
            app_profile.destination.display()
        );
        let mut existing_destination_files = gather_existing_files(&app_profile.destination)
            .await?
            .into_iter()
            .into_group_map_by(|entry| entry.path_inside_zip().to_owned());
        info!(
            "Found {} files in the destination ({})",
            existing_destination_files.len(),
            existing_destination_files.human_size()
        );

        info!("Gathering zip files from sources...");
        let (zips, zips_size) = get_zips::get_zips(&app_profile.sources).await?;
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
        info!("Partitioning entries by name...");
        let mut entries = entries
            .into_iter()
            .into_group_map_by(|entry| entry.path_inside_zip.clone());
        info!(
            "Found {} unique names used by {} entries ({})",
            entries.len(),
            entries.count(),
            entries.human_size()
        );

        let known_paths_in_zips: HashSet<PathInsideZip> = entries
            .keys()
            .chain(existing_destination_files.keys())
            .cloned()
            .collect();
        let mut to_audit = Vec::new();
        for path_in_zip in known_paths_in_zips {
            let existing_files = existing_destination_files
                .remove(&path_in_zip)
                .unwrap_or_default();
            let entries_for_path = entries.remove(&path_in_zip).unwrap_or_default();
            to_audit.push((path_in_zip, existing_files, entries_for_path));
        }

        let hasher_config = Arc::new(HasherConfig::new().hash_alg(HashAlg::Gradient));

        track_progress(
            to_audit,
            Duration::from_millis(500),
            |progress| info!("Enqueueing {progress}"),
            |progress| info!("Processing {progress}"),
            |_progress, elapsed| info!("Completed in {elapsed}"),
            move |(path_in_zip, existing_files, zip_entries)| {
                let hasher_config = hasher_config.clone();
                async move {
                    audit_path(
                        &path_in_zip,
                        existing_files,
                        zip_entries,
                        hasher_config,
                    )
                    .await
                    .wrap_err_with(|| format!("Failed to audit path {}", path_in_zip.display()))?;
                    Ok(())
                }
            },
            24,
        )
        .await?;

        Ok(())
    }
}

/// Consider that a user may has synced and since deleted the zip file.
/// We want to make sure that the destination contents doesn't disagree with the zip file contents, but we do not require the zip file entry to be present for all files in the destination.
/// Can validate the uncompressed bytes, crc hash, and image hash.
pub async fn audit_path(
    path_in_zip: &PathInsideZip,
    existing_files: Vec<ExistingFile>,
    zip_entries: Vec<ZipEntry>,
    hasher_config: Arc<HasherConfig>,
) -> Result<()> {
    if existing_files.is_empty() && zip_entries.is_empty() {
        warn!("No files found for path {}", path_in_zip.display());
        return Ok(());
    }

    let existing_files_set: HashSet<_> = existing_files
        .iter()
        .map(|entry| entry.path_inside_zip())
        .collect();
    let mut seen = HashSet::new();

    // All zip entries should be present in the destination
    for entry in &zip_entries {
        if !existing_files_set.contains(&entry.path_inside_zip) {
            warn!(
                "Missing file in destination for {}",
                entry.path_inside_zip.display()
            );
        } else {
            seen.insert(&entry.path_inside_zip);
        }
    }

    // audit crc32 and uncompressed size and image hash

    

    Ok(())
}
