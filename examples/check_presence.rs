// filepath: g:\Programming\Repos\meta-takeout\examples\check_presence.rs
use eyre::Result;
use eyre::eyre;
use positioned_io::RandomAccessFile;
use rc_zip_tokio::ReadZip;
use std::collections::HashMap;
use std::collections::HashSet;
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

    // Collect zip paths with modification times
    let mut zip_paths: Vec<(PathBuf, SystemTime)> = Vec::new();
    for dir in &dirs {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path
                .extension()
                .and_then(|s| s.to_str())
                .map_or(false, |ext| ext.eq_ignore_ascii_case("zip"))
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
    // Separate zip paths by export directory
    let mut older_paths: Vec<PathBuf> = Vec::new();
    let mut new_paths: Vec<PathBuf> = Vec::new();
    for (path, _) in &zip_paths {
        if path.starts_with(existing_zip_dir) {
            older_paths.push(path.clone());
        } else if path.starts_with(new_zip_dir) {
            new_paths.push(path.clone());
        }
    }
    println!(
        "Found {} old ZIPs and {} new ZIPs",
        older_paths.len(),
        new_paths.len()
    );

    // Parallel scan: build map per zip of dir -> set of file names
    let mut tasks = JoinSet::new();
    for (path, _) in &zip_paths {
        let path_clone = path.clone();
        tasks.spawn(async move {
            let mut dir_map: HashMap<String, HashSet<String>> = HashMap::new();
            let f = Arc::new(RandomAccessFile::open(&path_clone)?);
            let archive = f.read_zip().await?;
            for entry in archive.entries() {
                let name = entry
                    .sanitized_name()
                    .ok_or(eyre!("Invalid entry in {:?}", path_clone))?;
                if name.ends_with('/') {
                    continue;
                }
                let pb: PathBuf = PathBuf::from(name);
                let dir = pb
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "".to_string());
                let file = pb.file_name().unwrap().to_string_lossy().to_string();
                dir_map.entry(dir).or_default().insert(file);
            }
            Ok::<_, eyre::Report>((path_clone, dir_map))
        });
    }

    // Collect results
    let mut per_zip: HashMap<PathBuf, HashMap<String, HashSet<String>>> = HashMap::new();
    while let Some(res) = tasks.join_next().await {
        let (path, map) = res??;
        per_zip.insert(path, map);
    }

    // Combine per-export directory maps
    let mut older_map: HashMap<String, HashSet<String>> = HashMap::new();
    for path in older_paths {
        if let Some(map) = per_zip.get(&path) {
            for (dir, files) in map {
                older_map
                    .entry(dir.clone())
                    .or_default()
                    .extend(files.iter().cloned());
            }
        }
    }
    let mut new_map: HashMap<String, HashSet<String>> = HashMap::new();
    for path in new_paths {
        if let Some(map) = per_zip.get(&path) {
            for (dir, files) in map {
                new_map
                    .entry(dir.clone())
                    .or_default()
                    .extend(files.iter().cloned());
            }
        }
    }

    // Compare older exports to the newest
    println!("Comparing old export to newest export directories");
    for (dir, old_files) in &older_map {
        if let Some(new_files) = new_map.get(dir) {
            let missing: Vec<_> = old_files.difference(new_files).cloned().collect();
            if !missing.is_empty() {
                println!("Directory '{}' missing {} files", dir, missing.len());
            }
        }
    }

    let total_old: usize = older_map.values().map(|s| s.len()).sum();
    let total_new: usize = new_map.values().map(|s| s.len()).sum();
    println!("Total files in old export: {}", total_old);
    println!("Total files in new export: {}", total_new);

    Ok(())
}
