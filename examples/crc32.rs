// filepath: g:\Programming\Repos\meta-takeout\examples\check_crc32.rs
use crc32fast::Hasher as Crc32Hasher;
use eyre::Result;
use eyre::eyre;
use positioned_io::RandomAccessFile;
use rand::seq::IteratorRandom;
use rand::thread_rng;
use rc_zip_tokio::ReadZip;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() -> Result<()> {
    // Directories containing zip files
    let existing_zip_dir = r"C:\Users\TeamD\OneDrive\Documents\Backups\meta\facebook 2024-06";
    let new_zip_dir = r"C:\Users\TeamD\Downloads\facebookexport";
    let dirs = [existing_zip_dir, new_zip_dir];

    // Collect all zip file paths with modification times
    let mut zip_paths: Vec<(PathBuf, SystemTime)> = Vec::new();
    for dir in &dirs {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path
                .extension()
                .and_then(|s| s.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
            {
                let meta = std::fs::metadata(&path)?;
                let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                zip_paths.push((path, modified));
            }
        }
    }
    if zip_paths.is_empty() {
        eyre::bail!("No zip files found in specified directories");
    }
    println!("Found {} zip files", zip_paths.len());

    // Spawn a task per zip to build local map and sample entries for validation
    let mut tasks = JoinSet::new();
    for (path, modified) in &zip_paths {
        let path_clone = path.clone();
        let modified_time = *modified;
        tasks.spawn(async move {
            let mut local_map: HashMap<String, (u32, SystemTime, u64)> = HashMap::new();
            let mut names: Vec<String> = Vec::new();
            let f = Arc::new(RandomAccessFile::open(&path_clone)?);
            let archive = f.read_zip().await?;
            for entry in archive.entries() {
                let name = entry
                    .sanitized_name()
                    .ok_or(eyre!("Invalid entry in {:?}", path_clone))?;
                if name.ends_with('/') {
                    continue;
                }
                let key = name.to_owned();
                let crc = entry.crc32;
                let size = entry.uncompressed_size;
                local_map.insert(key.clone(), (crc, modified_time, size));
                names.push(key);
            }
            // Sample entries for CRC validation
            const CHECK_COUNT: usize = 100;
            let sample_keys: Vec<String> = {
                let mut rng = thread_rng();
                names.into_iter().choose_multiple(&mut rng, CHECK_COUNT)
            };
            let mut local_valid: Vec<bool> = Vec::new();
            for key in sample_keys {
                if let Some(entry) = archive.by_name(&key) {
                    let data = entry.bytes().await?;
                    let mut hasher = Crc32Hasher::new();
                    hasher.update(&data);
                    let calc = hasher.finalize();
                    local_valid.push(calc == entry.crc32);
                }
            }
            Ok::<_, eyre::Report>((local_map, local_valid))
        });
    }

    // Stats by file extension
    #[derive(Default)]
    struct Stats {
        count: usize,
        matches: usize,
        not_matches: usize,
        zeros: usize,
        newer_larger: usize,
        newer_smaller: usize,
        newer_equal: usize,
    }
    let mut ext_stats: HashMap<String, Stats> = HashMap::new();
    // Global entry history: key -> Vec<(crc, modified, size)>
    let mut global_map: HashMap<String, Vec<(u32, SystemTime, u64)>> = HashMap::new();
    let mut total_validations = 0;
    let mut total_pass = 0;
    let mut total_fail = 0;
    while let Some(res) = tasks.join_next().await {
        let (local_map, local_valid) = res??;
        for (key, (crc, modified, size)) in local_map {
            // Determine extension
            let ext = PathBuf::from(&key)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let stats = ext_stats.entry(ext.clone()).or_default();
            // Count unique entries per extension
            let history = global_map.entry(key.clone()).or_default();
            if history.is_empty() {
                stats.count += 1;
            }
            // Zero CRC
            if crc == 0 {
                stats.zeros += 1;
            }
            // Compare against previous entries for this key
            for &(prev_crc, prev_mod, prev_size) in history.iter() {
                if crc != 0 && prev_crc != 0 {
                    if crc == prev_crc {
                        stats.matches += 1;
                    } else {
                        stats.not_matches += 1;
                    }
                }
                // newer size comparisons with equal
                if modified > prev_mod {
                    if size > prev_size {
                        stats.newer_larger += 1;
                    } else if size < prev_size {
                        stats.newer_smaller += 1;
                    } else {
                        stats.newer_equal += 1;
                    }
                } else if prev_mod > modified {
                    if prev_size > size {
                        stats.newer_larger += 1;
                    } else if prev_size < size {
                        stats.newer_smaller += 1;
                    } else {
                        stats.newer_equal += 1;
                    }
                }
            }
            // Record this instance
            history.push((crc, modified, size));
        }
        // Collect validation results
        for ok in local_valid {
            total_validations += 1;
            if ok {
                total_pass += 1;
            } else {
                total_fail += 1;
            }
        }
    }
    // Print final stats per extension, sorted by count descending
    let mut ext_stats_vec: Vec<_> = ext_stats.iter().collect();
    ext_stats_vec.sort_by(|a, b| b.1.count.cmp(&a.1.count));
    println!("Stats by extension:");
    for (ext, s) in ext_stats_vec {
        println!(
            "{}: count={} | CRC(matches={} mismatches={} zeros={}) | SIZE(>={} <={} =={})",
            ext,
            s.count,
            s.matches,
            s.not_matches,
            s.zeros,
            s.newer_larger,
            s.newer_smaller,
            s.newer_equal
        );
    }

    // Validation summary
    println!("\nValidation summary:");
    println!("  Checked entries: {total_validations}");
    println!("  Passes:          {total_pass}");
    println!("  Failures:        {total_fail}");

    Ok(())
}
