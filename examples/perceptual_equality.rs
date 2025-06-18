use eyre::OptionExt;
use eyre::Result;
use eyre::eyre;
use holda::Holda;
use img_hash::HashAlg;
use img_hash::Hasher as ImgHasher;
use img_hash::HasherConfig;
use img_hash::ImageHash;
use positioned_io::RandomAccessFile;
use rc_zip_tokio::ReadZip;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task::JoinSet;
use tracing::error;
use tracing::info;
use tracing::warn;

// Define PathToZip and PathInsideZip structs
#[derive(Holda)]
#[holda(NoDisplay)]
pub struct PathToZip {
    inner: PathBuf,
}
impl AsRef<Path> for PathToZip {
    fn as_ref(&self) -> &Path {
        self.inner.as_ref()
    }
}
impl std::fmt::Display for PathToZip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.display())
    }
}

#[derive(Holda)]
#[holda(NoDisplay)]
pub struct PathInsideZip {
    inner: PathBuf,
}
impl AsRef<Path> for PathInsideZip {
    fn as_ref(&self) -> &Path {
        self.inner.as_ref()
    }
}
impl std::fmt::Display for PathInsideZip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.display())
    }
}

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
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_max_level(tracing::Level::INFO)
        .init();

    let existing_zip_dir = r"C:\Users\TeamD\OneDrive\Documents\Backups\meta\facebook 2024-06";
    let new_zip_dir = r"C:\Users\TeamD\Downloads\facebookexport";
    let dirs = [existing_zip_dir, new_zip_dir];

    let mut zip_paths_bufs: Vec<PathBuf> = Vec::new();
    for dir_str in &dirs {
        let dir_path = Path::new(dir_str);
        if !dir_path.exists() {
            warn!("Directory {} does not exist, skipping.", dir_path.display());
            continue;
        }
        if !dir_path.is_dir() {
            warn!("Path {} is not a directory, skipping.", dir_path.display());
            continue;
        }
        match std::fs::read_dir(dir_path) {
            Ok(entries) => {
                for entry_result in entries {
                    match entry_result {
                        Ok(entry) => {
                            if entry
                                .path()
                                .extension()
                                .map_or(false, |e| e.eq_ignore_ascii_case("zip"))
                            {
                                zip_paths_bufs.push(entry.path());
                            }
                        }
                        Err(e) => error!("Failed to read entry in {}: {}", dir_path.display(), e),
                    }
                }
            }
            Err(e) => error!("Failed to read directory {}: {}", dir_path.display(), e),
        }
    }

    let zip_paths: Vec<PathToZip> = zip_paths_bufs.into_iter().map(PathToZip::from).collect();

    if zip_paths.is_empty() {
        error!("No zip files found or accessible in the specified directories. Exiting.");
        return Ok(());
    }
    info!("Found {} zip files to process.", zip_paths.len());

    // Phase 1: Grab all entries and their raw info
    let mut entry_map: HashMap<PathInsideZip, Vec<RawEntryInfo>> = HashMap::new();
    for zip_path_obj in &zip_paths {
        info!("Scanning zip: {}", zip_path_obj);
        let f = match RandomAccessFile::open(&zip_path_obj.inner) {
            Ok(file) => Arc::new(file),
            Err(e) => {
                error!("Failed to open zip {}: {}", zip_path_obj, e);
                continue;
            }
        };
        let archive = match f.read_zip().await {
            Ok(arch) => arch,
            Err(e) => {
                error!("Failed to read zip {}: {}", zip_path_obj, e);
                continue;
            }
        };

        for entry in archive.entries() {
            // Check if entry is a directory by looking at its name (common practice)
            // or by specific flags if available (rc_zip might have a method like entry.is_directory() or similar)
            // For now, assuming directories might end with a '/'
            let name = entry
                .sanitized_name()
                .ok_or_eyre(eyre!("Invalid entry name in {}", zip_path_obj))?;
            let name = entry
                .sanitized_name()
                .ok_or_eyre(eyre!("Invalid entry name in {}", zip_path_obj))?;
            if name.ends_with('/') {
                continue;
            }

            let name_buf = match entry.sanitized_name() {
                Some(name) => name,
                None => {
                    warn!("Skipping entry with invalid name in {}", zip_path_obj);
                    continue;
                }
            };
            let path_inside_zip = PathInsideZip::from(PathBuf::from(name_buf));

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
        let entry_path_display = entry_path_obj.to_string(); // For logging

        let extension = entry_path_obj
            .inner
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
            let task_entry_path_buf = entry_path_obj.inner.clone(); // Path inside zip
            let task_compressed_size = raw_info.compressed_size;

            tasks.spawn(async move {
                let f = match RandomAccessFile::open(&task_zip_path.inner) {
                    Ok(file) => Arc::new(file),
                    Err(e) => return Err(eyre!("Zip {}: Failed to open: {}", task_zip_path, e)),
                };
                let archive = match f.read_zip().await {
                    Ok(arch) => arch,
                    Err(e) => {
                        return Err(eyre!(
                            "Zip {}: Failed to read archive: {}",
                            task_zip_path,
                            e
                        ));
                    }
                };

                // Find the specific entry again
                let entry_opt = archive.entries().find(|e| {
                    e.sanitized_name()
                        .map_or(false, |n| PathBuf::from(n) == task_entry_path_buf)
                });

                if let Some(entry) = entry_opt {
                    let data: Vec<u8> = match entry.bytes().await {
                        Ok(d) => d,
                        Err(e) => {
                            return Err(eyre!(
                                "Entry '{}' in Zip {}: Failed to read bytes: {}",
                                task_entry_path_buf.display(),
                                task_zip_path,
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
                        task_zip_path
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
                smallest_instance.zip_path,
                smallest_instance.compressed_size,
                MAX_HAMMING_DISTANCE
            );
        } else {
            println!(
                "Entry '{}': Perceptual MISMATCH found (some pairs > dist {}). Details:",
                entry_path_display, MAX_HAMMING_DISTANCE
            );
            for instance in image_instances_results {
                println!(
                    "  - File: '{}', Hash: {}, Size: {} bytes",
                    instance.zip_path,
                    instance.hash.to_base64(),
                    instance.compressed_size
                );
            }
        }
    }

    info!("Perceptual equality check finished.");
    Ok(())
}
