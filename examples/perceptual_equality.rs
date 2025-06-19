use eyre::Context;
use eyre::OptionExt;
use eyre::Result;
use eyre::eyre;
use img_hash::HashAlg;
use img_hash::Hasher as ImgHasher;
use img_hash::HasherConfig;
use img_hash::ImageHash;
use positioned_io::RandomAccessFile;
use rc_zip_tokio::ReadZip;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use thrumzip::get_zips::get_zips;
use thrumzip::path_inside_zip::PathInsideZip;
use thrumzip::path_to_zip::PathToZip;
use thrumzip::state::profiles::Profile;
use tokio::task::JoinSet;
use tracing::Level;
use tracing::error;
use tracing::info;
use tracing::warn;

const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp"];
// Max Hamming distance for two images to be considered "similar".
// For a 64-bit hash, a distance of 0 means 100% hash identity.
// (64-0)/64 = 1.0 (100% similarity)
// (64-1)/64 = 0.984375 (98.4375% similarity)
// To meet >= 99% similarity for a 64-bit hash, distance must be 0.
const MAX_HAMMING_DISTANCE: u32 = 0;

#[derive(Debug)]
struct ImageInstance {
    zip_path: PathToZip,
    hash: ImageHash,
    compressed_size: u64,
}

#[derive(Clone, Debug)]
struct RawEntryInfo {
    zip_path: PathToZip,
    compressed_size: u64,
}

fn create_image_hasher() -> ImgHasher {
    HasherConfig::new().hash_alg(HashAlg::Gradient).to_hasher()
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install().wrap_err("Failed to install color_eyre")?;
    thrumzip::init_tracing::init_tracing(Level::INFO);
    // let profile = thrumzip::state::profiles::Profiles::load_and_get_active_profile().await?;
    let profile = Profile::new_example();

    // Collect zip files from both directories
    let (zip_paths, _) = get_zips(&profile.sources).await?;
    if zip_paths.is_empty() {
        eyre::bail!("No zip files found in {:?}", profile.sources);
    }

    if zip_paths.is_empty() {
        error!("No zip files found or accessible in the specified directories. Exiting.");
        return Ok(());
    }
    info!("Found {} zip files to process.", zip_paths.len());

    // Phase 1: Grab all entries and their raw info
    let mut entry_map: HashMap<PathInsideZip, Vec<RawEntryInfo>> = HashMap::new();
    for zip_path_obj in &zip_paths {
        info!("Scanning zip: {}", zip_path_obj.display());
        let f = match RandomAccessFile::open(zip_path_obj) {
            Ok(file) => Arc::new(file),
            Err(e) => {
                error!("Failed to open zip {}: {}", zip_path_obj.display(), e);
                continue;
            }
        };
        let archive = match f.read_zip().await {
            Ok(arch) => arch,
            Err(e) => {
                error!("Failed to read zip {}: {}", zip_path_obj.display(), e);
                continue;
            }
        };

        for entry in archive.entries() {
            // Check if entry is a directory by looking at its name (common practice)
            // or by specific flags if available (rc_zip might have a method like entry.is_directory() or similar)
            // For now, assuming directories might end with a '/'
            let name = entry
                .sanitized_name()
                .ok_or_eyre(eyre!("Invalid entry name in {}", zip_path_obj.display()))?;
            if name.ends_with('/') {
                continue;
            }

            let name_buf = match entry.sanitized_name() {
                Some(name) => name,
                None => {
                    warn!(
                        "Skipping entry with invalid name in {}",
                        zip_path_obj.display()
                    );
                    continue;
                }
            };
            let path_inside_zip = PathInsideZip::new(Arc::new(PathBuf::from(name_buf)));

            let raw_info = RawEntryInfo {
                zip_path: zip_path_obj.clone(),
                compressed_size: entry.compressed_size,
            };
            entry_map.entry(path_inside_zip).or_default().push(raw_info);
        }
    }
    info!(
        "Finished scanning all zip files. Found {} unique entry paths.",
        entry_map.len()
    );

    // Phase 2: Process duplicates for perceptual hashing
    for (entry_path_obj, raw_infos) in entry_map.into_iter().filter(|(_, v)| v.len() > 1) {
        let entry_path_display = entry_path_obj.display(); // For logging

        let extension = entry_path_obj
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !IMAGE_EXTENSIONS.contains(&extension.as_str()) {
            continue; // Not an image extension we're interested in
        }

        info!(
            "Processing potential image entry: {} ({} instances)",
            entry_path_display,
            raw_infos.len()
        );

        let mut image_instances_results: Vec<ImageInstance> = Vec::new();
        let mut tasks = JoinSet::new();

        for raw_info in raw_infos {
            let task_zip_path = raw_info.zip_path.clone();
            let task_entry_path_buf = entry_path_obj.clone(); // Path inside zip
            let task_compressed_size = raw_info.compressed_size;

            tasks.spawn(async move {
                let f = match RandomAccessFile::open(&task_zip_path) {
                    Ok(file) => Arc::new(file),
                    Err(e) => {
                        return Err(eyre!(
                            "Zip {}: Failed to open: {}",
                            task_zip_path.display(),
                            e
                        ));
                    }
                };
                let archive = match f.read_zip().await {
                    Ok(arch) => arch,
                    Err(e) => {
                        return Err(eyre!(
                            "Zip {}: Failed to read archive: {}",
                            task_zip_path.display(),
                            e
                        ));
                    }
                };

                // Find the specific entry again
                let entry_opt = archive.entries().find(|e| {
                    e.sanitized_name()
                        .is_some_and(|n| PathBuf::from(n) == **task_entry_path_buf)
                });

                if let Some(entry) = entry_opt {
                    let data: Vec<u8> = match entry.bytes().await {
                        Ok(d) => d,
                        Err(e) => {
                            return Err(eyre!(
                                "Entry '{}' in Zip {}: Failed to read bytes: {}",
                                task_entry_path_buf.display(),
                                task_zip_path.display(),
                                e
                            ));
                        }
                    };

                    let image_data = match image::load_from_memory(&data) {
                        Ok(img) => img,
                        Err(_e) => {
                            return Ok(None);
                        }
                    };
                    let hasher = create_image_hasher(); // Create a new hasher for each task
                    let hash = hasher.hash_image(&image_data);
                    Ok(Some(ImageInstance {
                        zip_path: task_zip_path,
                        hash,
                        compressed_size: task_compressed_size,
                    }))
                } else {
                    Err(eyre!(
                        "Entry '{}' unexpectedly not found in zip {} during hashing phase.",
                        task_entry_path_buf.display(),
                        task_zip_path.display()
                    ))
                }
            });
        }

        while let Some(join_result) = tasks.join_next().await {
            match join_result {
                Ok(Ok(Some(instance))) => image_instances_results.push(instance),
                Ok(Ok(None)) => { /* Successfully determined it's not a hashable image */ }
                Ok(Err(e)) => error!(
                    "Task error processing an instance of entry '{}': {:?}",
                    entry_path_display, e
                ),
                Err(e) => error!(
                    "JoinError for task processing entry '{}': {:?}",
                    entry_path_display, e
                ),
            }
        }

        if image_instances_results.len() < 2 {
            if !image_instances_results.is_empty() {
                info!(
                    "Entry '{}': Not enough successfully processed image instances to compare (need at least 2, got {}).",
                    entry_path_display,
                    image_instances_results.len()
                );
            }
            continue;
        }

        let mut are_all_mutually_similar = true;
        for i in 0..image_instances_results.len() {
            for j in (i + 1)..image_instances_results.len() {
                let inst_i = &image_instances_results[i];
                let inst_j = &image_instances_results[j];
                let dist = inst_i.hash.dist(&inst_j.hash);
                if dist > MAX_HAMMING_DISTANCE {
                    are_all_mutually_similar = false;
                    break;
                }
            }
            if !are_all_mutually_similar {
                break;
            }
        }

        if are_all_mutually_similar {
            let smallest_instance = image_instances_results
                .iter()
                .min_by_key(|instance| instance.compressed_size)
                .unwrap();

            println!(
                "Entry '{}': Perceptually SIMILAR across {} files. Smallest: '{}' ({} bytes). (Max dist: {})",
                entry_path_display,
                image_instances_results.len(),
                smallest_instance.zip_path.display(),
                smallest_instance.compressed_size,
                MAX_HAMMING_DISTANCE
            );
        } else {
            println!(
                "Entry '{entry_path_display}': Perceptual MISMATCH found (some pairs > dist {MAX_HAMMING_DISTANCE}). Details:"
            );
            for instance in image_instances_results {
                println!(
                    "  - File: '{}', Hash: {}, Size: {} bytes",
                    instance.zip_path.display(),
                    instance.hash.to_base64(),
                    instance.compressed_size
                );
            }
        }
    }

    info!("Perceptual equality check finished.");
    Ok(())
}
