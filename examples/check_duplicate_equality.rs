use eyre::Context;
use eyre::Result;
use eyre::eyre;
use positioned_io::RandomAccessFile;
use rc_zip_tokio::ReadZip;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::path::PathBuf;
use std::sync::Arc;
use thrumzip::get_zips::get_zips;
use thrumzip::path_inside_zip::PathInsideZip;
use thrumzip::path_to_zip::PathToZip;
use thrumzip::state::profiles::Profile;
use tracing::Level;

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
    if zip_paths.len() < 2 {
        eyre::bail!("Need at least two zip files to compare");
    }

    // Map each entry name to the list of zip archives containing it
    let mut entry_map: HashMap<PathInsideZip, Vec<PathToZip>> = HashMap::new();
    for zip in &zip_paths {
        let f = Arc::new(RandomAccessFile::open(&zip)?);
        let archive = f.read_zip().await?;
        for entry in archive.entries() {
            let name_buf = entry
                .sanitized_name()
                .ok_or_else(|| eyre!("Invalid entry name"))?;
            let name = PathInsideZip::new(Arc::new(name_buf.into()));
            entry_map.entry(name).or_default().push(zip.clone());
        }
    }

    // Check entries that appear in multiple zip files
    for (name, zips) in entry_map.into_iter().filter(|(_, v)| v.len() > 1) {
        println!("Checking entry {}", name.display());
        let mut hashes = Vec::new();
        for zip in &zips {
            let f = Arc::new(RandomAccessFile::open(&zip)?);
            let archive = f.read_zip().await?;
            // this is super inefficient
            if let Some(entry) = archive.entries().find(|e| {
                e.sanitized_name()
                    .map(|n| PathBuf::from(n) == **name)
                    .unwrap_or(false)
            }) {
                let data = entry.bytes().await?;
                let mut hasher = DefaultHasher::new();
                hasher.write(&data);
                hashes.push((zip, hasher.finish()));
            }
        }
        let unique_hashes: HashSet<u64> = hashes.iter().map(|(_, h)| *h).collect();
        if unique_hashes.len() > 1 {
            println!("  MISMATCH for entry {}", name.display());
            for (zip, h) in hashes {
                println!("    {} => {:016x}", zip.display(), h);
            }
        } else {
            println!("  identical across {} files", zips.len());
        }
    }

    Ok(())
}
