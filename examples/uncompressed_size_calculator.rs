// filepath: g:\Programming\Repos\meta-takeout\examples\uncompressed_size_calculator.rs
use eyre::Result;
use eyre::eyre;
use positioned_io::RandomAccessFile;
use rc_zip_tokio::ReadZip;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::task::JoinSet;

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    match bytes {
        b if b >= GB => format!("{:.2} GB", b as f64 / GB as f64),
        b if b >= MB => format!("{:.2} MB", b as f64 / MB as f64),
        b if b >= KB => format!("{:.2} KB", b as f64 / KB as f64),
        _ => format!("{bytes} B"),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Directories containing zip files
    let existing_zip_dir = r"C:\Users\TeamD\OneDrive\Documents\Backups\meta\facebook 2024-06";
    let new_zip_dir = r"C:\Users\TeamD\Downloads\facebookexport";
    let dirs = [existing_zip_dir, new_zip_dir];

    // Collect zip paths with modification times
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

    // Parallel scan: spawn a task for each zip to build its local map
    let mut tasks = JoinSet::new();
    for (path, modified) in zip_paths.clone() {
        tasks.spawn(async move {
            let mut local: HashMap<String, (SystemTime, u64)> = HashMap::new();
            let f = Arc::new(RandomAccessFile::open(&path)?);
            let archive = f.read_zip().await?;
            for entry in archive.entries() {
                let name = entry
                    .sanitized_name()
                    .ok_or(eyre!("Invalid entry in {:?}", path))?;
                if name.ends_with('/') {
                    continue;
                }
                let key = name.to_owned();
                let size = entry.uncompressed_size;
                if let Some((prev_mod, _)) = local.get(&key) {
                    if *prev_mod >= modified {
                        continue;
                    }
                }
                local.insert(key, (modified, size));
            }
            Ok::<_, eyre::Report>(local)
        });
    }

    // Merge per-zip maps selecting newest entries
    let mut entry_map: HashMap<String, (SystemTime, u64)> = HashMap::new();
    while let Some(res) = tasks.join_next().await {
        let local_map = res??;
        for (key, (mod_time, size)) in local_map {
            match entry_map.get(&key) {
                Some((prev, _)) if *prev >= mod_time => continue,
                _ => {
                    entry_map.insert(key, (mod_time, size));
                }
            }
        }
    }

    // Sum uncompressed sizes for latest variants
    let total: u64 = entry_map.values().map(|&(_, size)| size).sum();
    println!(
        "Total uncompressed size: {} ({})",
        total,
        format_bytes(total)
    );
    Ok(())
}
