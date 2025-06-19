// this is too slow.

use eyre::Result;
use eyre::eyre;
use holda::Holda;
use positioned_io::RandomAccessFile;
use rc_zip_tokio::ReadZip;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

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

#[tokio::main]
async fn main() -> Result<()> {
    // let args: Vec<String> = std::env::args().collect();
    // if args.len() != 3 {
    //     eprintln!("Usage: {} <zip_dir1> <zip_dir2>", args[0]);
    //     std::process::exit(1);
    // }
    // let dirs = vec![&args[1], &args[2]];
    let existing_zip_dir = r"C:\Users\TeamD\OneDrive\Documents\Backups\meta\facebook 2024-06";
    let new_zip_dir = r"C:\Users\TeamD\Downloads\facebookexport";
    let dirs = [existing_zip_dir, new_zip_dir];
    // Collect zip files from both directories
    let mut zip_paths: Vec<PathToZip> = Vec::new();
    for dir in &dirs {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            if entry
                .path()
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("zip"))
            {
                zip_paths.push(PathToZip {
                    inner: entry.path(),
                });
            }
        }
    }
    if zip_paths.len() < 2 {
        eyre::bail!("Need at least two zip files to compare");
    }

    // Map each entry name to the list of zip archives containing it
    let mut entry_map: HashMap<PathInsideZip, Vec<PathToZip>> = HashMap::new();
    for zip in &zip_paths {
        let f = Arc::new(RandomAccessFile::open(&zip.inner)?);
        let archive = f.read_zip().await?;
        for entry in archive.entries() {
            let name_buf = entry
                .sanitized_name()
                .ok_or_else(|| eyre!("Invalid entry name"))?;
            let name = PathInsideZip {
                inner: PathBuf::from(name_buf),
            };
            entry_map.entry(name).or_default().push(zip.clone());
        }
    }

    // Check entries that appear in multiple zip files
    for (name, zips) in entry_map.into_iter().filter(|(_, v)| v.len() > 1) {
        println!("Checking entry {}", name.inner.display());
        let mut hashes = Vec::new();
        for zip in &zips {
            let f = Arc::new(RandomAccessFile::open(&zip.inner)?);
            let archive = f.read_zip().await?;
            if let Some(entry) = archive.entries().find(|e| {
                e.sanitized_name()
                    .map(|n| PathBuf::from(n) == name.inner)
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
            println!("  MISMATCH for entry {}", name.inner.display());
            for (zip, h) in hashes {
                println!("    {} => {:016x}", zip.inner.display(), h);
            }
        } else {
            println!("  identical across {} files", zips.len());
        }
    }

    Ok(())
}
