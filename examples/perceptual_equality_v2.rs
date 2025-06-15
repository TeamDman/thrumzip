use eyre::Result;
use eyre::eyre;
use holda::Holda;
use image::load_from_memory;
use img_hash::HashAlg;
use img_hash::HasherConfig;
use img_hash::ImageHash;
use positioned_io::RandomAccessFile;
use rc_zip_tokio::ReadZip;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task::JoinSet;
use tracing::info;
use tracing::warn;
use std::time::{Instant, Duration};
use std::sync::{Mutex, Arc as StdArc};
use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
use humantime::format_duration;

#[derive(Holda)]
#[holda(NoDisplay)]
struct PathToZip {
    inner: PathBuf,
}
impl AsRef<Path> for PathToZip {
    fn as_ref(&self) -> &Path {
        &self.inner
    }
}

#[derive(Holda)]
#[holda(NoDisplay)]
struct PathInsideZip {
    inner: PathBuf,
}
impl AsRef<Path> for PathInsideZip {
    fn as_ref(&self) -> &Path {
        &self.inner
    }
}

/// Maximum Hamming distance threshold for perceptual difference
const MAX_HAMMING: u32 = 1;

/// Create a perceptual hasher
fn image_hasher() -> img_hash::Hasher {
    HasherConfig::new().hash_alg(HashAlg::Gradient).to_hasher()
}

/// Raw entry info: ZIP path and stored CRC32
struct RawInfo {
    zip: PathToZip,
    crc: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt().init();

    // Directories to scan
    let dirs = [
        r"C:\Users\TeamD\OneDrive\Documents\Backups\meta\facebook 2024-06",
        r"C:\Users\TeamD\Downloads\facebookexport",
    ];
    // Collect all ZIP file paths
    let mut zip_files = Vec::new();
    for d in &dirs {
        let dir = Path::new(d);
        if !dir.is_dir() {
            warn!("{} is not a directory, skipping", d);
            continue;
        }
        for entry in std::fs::read_dir(dir)? {
            let p = entry?.path();
            if p.extension()
                .and_then(|s| s.to_str())
                .map_or(false, |e| e.eq_ignore_ascii_case("zip"))
            {
                zip_files.push(PathToZip { inner: p });
            }
        }
    }
    if zip_files.is_empty() {
        eyre::bail!("No ZIP files found");
    }
    info!("Found {} ZIPs", zip_files.len());

    // Phase 1: scan entries and CRCs in parallel
    let image_exts = ["png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp"];
    let mut scan_set = JoinSet::new();
    for zip in &zip_files {
        let z = zip.clone();
        let image_exts = image_exts.clone();
        scan_set.spawn(async move {
            let f = Arc::new(RandomAccessFile::open(&z.inner)?);
            let arch = f.read_zip().await?;
            let mut list = Vec::new();
            for ent in arch.entries() {
                if let Some(name) = ent.sanitized_name() {
                    if !name.ends_with('/') {
                        let ext = Path::new(&name)
                            .extension()
                            .and_then(|e| e.to_str())
                            .map(|e| e.to_ascii_lowercase());
                        if let Some(ext) = ext {
                            if !image_exts.contains(&ext.as_str()) {
                                continue;
                            }
                        } else {
                            continue;
                        }
                        let key = PathInsideZip {
                            inner: PathBuf::from(name),
                        };
                        list.push((
                            key,
                            RawInfo {
                                zip: z.clone(),
                                crc: ent.crc32,
                            },
                        ));
                    }
                }
            }
            Ok::<_, eyre::Report>(list)
        });
    }
    let mut entry_map: HashMap<PathInsideZip, Vec<RawInfo>> = HashMap::new();
    while let Some(res) = scan_set.join_next().await {
        // log scan progress
        let remaining = scan_set.len();
        info!("Scanned batch, scan tasks remaining: {}", remaining);
        for (key, info) in res?? {
            entry_map.entry(key).or_default().push(info);
        }
    }

    // Filter entries where CRCs differ
    let diff_entries: Vec<_> = entry_map
        .into_iter()
        .filter(|(_, infos)| {
            let mut cs = HashSet::new();
            infos.iter().for_each(|i| {
                cs.insert(i.crc);
            });
            cs.len() > 1
        })
        .collect();
    info!("{} entries have differing CRCs", diff_entries.len());

    // Stats struct for tracking distances
    #[derive(Default, Clone)]
    struct DistStats {
        count: usize,
        sum: u64,
        min: u32,
        max: u32,
        values: Vec<u32>,
    }
    impl DistStats {
        fn new() -> Self {
            Self { count: 0, sum: 0, min: u32::MAX, max: 0, values: Vec::new() }
        }
        fn add(&mut self, d: u32) {
            self.count += 1;
            self.sum += d as u64;
            self.min = self.min.min(d);
            self.max = self.max.max(d);
            self.values.push(d);
        }
        fn mean(&self) -> f64 {
            if self.count == 0 { 0.0 } else { self.sum as f64 / self.count as f64 }
        }
        fn stddev(&self) -> f64 {
            if self.count == 0 { return 0.0; }
            let mean = self.mean();
            let var = self.values.iter().map(|&v| {
                let d = v as f64 - mean;
                d*d
            }).sum::<f64>() / self.count as f64;
            var.sqrt()
        }
    }

    let total_batches = diff_entries.len();
    let processed_batches = StdArc::new(AtomicUsize::new(0));
    let exceeding_names = StdArc::new(Mutex::new(HashSet::new()));
    let stats_exceed = StdArc::new(Mutex::new(DistStats::new()));
    let stats_ok = StdArc::new(Mutex::new(DistStats::new()));
    let start = Instant::now();
    let iter_count = StdArc::new(AtomicUsize::new(0));

    // Stats printer thread
    {
        let processed_batches = processed_batches.clone();
        let exceeding_names = exceeding_names.clone();
        let stats_exceed = stats_exceed.clone();
        let stats_ok = stats_ok.clone();
        let start = start.clone();
        let iter_count = iter_count.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(1));
                let elapsed = start.elapsed().as_secs_f64();
                let done = processed_batches.load(Ordering::Relaxed);
                let total = total_batches;
                let eta = if done > 0 {
                    let secs = (elapsed / done as f64) * (total as f64 - done as f64);
                    Some(Duration::from_secs_f64(secs.max(0.0)))
                } else { None };
                let iters = iter_count.swap(0, Ordering::Relaxed);
                let ex = exceeding_names.lock().unwrap().len();
                let stats_ex = stats_exceed.lock().unwrap().clone();
                let stats_ok = stats_ok.lock().unwrap().clone();
                println!("[STATS] {}/{} batches, {:.2} it/s, ETA {}, exceeding: {}", done, total, iters as f64, eta.map(|d| format_duration(d).to_string()).unwrap_or("--".to_string()), ex);
                if stats_ex.count > 0 {
                    println!("  >exceed: min {}, max {}, mean {:.2}, stddev {:.2}, count {}", stats_ex.min, stats_ex.max, stats_ex.mean(), stats_ex.stddev(), stats_ex.count);
                }
                if stats_ok.count > 0 {
                    println!("  <=ok:    min {}, max {}, mean {:.2}, stddev {:.2}, count {}", stats_ok.min, stats_ok.max, stats_ok.mean(), stats_ok.stddev(), stats_ok.count);
                }
            }
        });
    }

    // Phase 2: process each entry batch, compute perceptual hashes and distances per group
    for (pi, infos) in diff_entries {
        iter_count.fetch_add(1, Ordering::Relaxed);
        // info!("Processing perceptual hashes for {}", pi.display());
        // remember count before moving infos
        let infos_count = infos.len();
        let mut group_set = JoinSet::new();
        for info in infos {
            let pi = pi.clone();
            let zi = info.zip.clone();
            group_set.spawn(async move {
                let f = Arc::new(RandomAccessFile::open(&zi.inner)?);
                let arch = f.read_zip().await?;
                let ent = arch
                    .by_name(pi.inner.to_string_lossy().as_ref())
                    .ok_or(eyre!("Missing {} in {}", pi.display(), zi.display()))?;
                let data = ent.bytes().await?;
                let img = load_from_memory(&data)?;
                let hash = image_hasher().hash_image(&img);
                Ok::<_, eyre::Report>((zi, hash))
            });
        }

        // Collect hashed results as they complete
        let mut results: Vec<(PathToZip, ImageHash)> = Vec::with_capacity(infos_count);
        while let Some(res) = group_set.join_next().await {
            let (zi, h) = res??;
            // info!("Hashed {} from {}", pi.display(), zi.display());
            results.push((zi, h));
        }

        // Compute and print distances for this entry batch
        let n = results.len();
        let mut local_exceed = false;
        for i in 0..n {
            for j in (i + 1)..n {
                let (_z1, h1) = &results[i];
                let (_z2, h2) = &results[j];
                let d = h1.dist(h2);
                if d > MAX_HAMMING {
                    local_exceed = true;
                    let mut stats = stats_exceed.lock().unwrap();
                    stats.add(d);
                } else {
                    let mut stats = stats_ok.lock().unwrap();
                    stats.add(d);
                }
            }
        }
        if local_exceed {
            exceeding_names.lock().unwrap().insert(pi.inner.clone());
        }
        processed_batches.fetch_add(1, Ordering::Relaxed);
    }

    Ok(())
}
