use eyre::Context;
use eyre::Result;
use eyre::eyre;
use humansize::DECIMAL;
use humansize::format_size;
use humantime::format_duration;
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
use std::sync::Arc as StdArc;
use std::sync::Mutex;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use thrumzip::get_zips::get_zips;
use thrumzip::path_inside_zip::PathInsideZip;
use thrumzip::path_to_zip::PathToZip;
use thrumzip::state::profiles::Profile;
use tokio::task::JoinSet;
use tracing::Level;
use tracing::info;

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
    size: u64,
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

    // Phase 1: scan entries and CRCs in parallel
    let image_exts = ["png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp"];
    let mut scan_set = JoinSet::new();
    for zip in &zip_paths {
        let zip_file_path = zip.clone();
        scan_set.spawn(async move {
            let f = Arc::new(RandomAccessFile::open(&zip_file_path)?);
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
                        let key = PathInsideZip::new(Arc::new(PathBuf::from(name)));
                        list.push((
                            key,
                            RawInfo {
                                zip: zip_file_path.clone(),
                                crc: ent.crc32,
                                size: ent.uncompressed_size,
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
            Self {
                count: 0,
                sum: 0,
                min: u32::MAX,
                max: 0,
                values: Vec::new(),
            }
        }
        fn add(&mut self, d: u32) {
            self.count += 1;
            self.sum += d as u64;
            self.min = self.min.min(d);
            self.max = self.max.max(d);
            self.values.push(d);
        }
        fn mean(&self) -> f64 {
            if self.count == 0 {
                0.0
            } else {
                self.sum as f64 / self.count as f64
            }
        }
        fn stddev(&self) -> f64 {
            if self.count == 0 {
                return 0.0;
            }
            let mean = self.mean();
            let var = self
                .values
                .iter()
                .map(|&v| {
                    let d = v as f64 - mean;
                    d * d
                })
                .sum::<f64>()
                / self.count as f64;
            var.sqrt()
        }
    }

    let total_batches = diff_entries.len();
    // Compute total bytes to process
    let total_bytes: u64 = diff_entries
        .iter()
        .flat_map(|(_, infos)| infos.iter().map(|i| i.size))
        .sum();
    info!(
        "Total bytes to process: {}",
        format_size(total_bytes, DECIMAL)
    );
    let processed_bytes = StdArc::new(AtomicU64::new(0));
    let processed_batches = StdArc::new(AtomicUsize::new(0));
    let exceeding_names = StdArc::new(Mutex::new(HashSet::new()));
    let stats_exceed = StdArc::new(Mutex::new(DistStats::new()));
    let stats_ok = StdArc::new(Mutex::new(DistStats::new()));
    let start = Instant::now();
    let iter_count = StdArc::new(AtomicUsize::new(0));

    // Stats printer thread with byte-rate ETA
    {
        // clone required counters
        let processed_batches = processed_batches.clone();
        let processed_bytes = processed_bytes.clone();
        let exceeding_names = exceeding_names.clone();
        let stats_exceed = stats_exceed.clone();
        let stats_ok = stats_ok.clone();
        let iter_count = iter_count.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(1));
                let elapsed = start.elapsed().as_secs_f64();
                let done_batches = processed_batches.load(Ordering::Relaxed);
                let done_bytes = processed_bytes.load(Ordering::Relaxed);
                let rate_bps = if elapsed > 0.0 {
                    done_bytes as f64 / elapsed
                } else {
                    0.0
                };
                let remaining_bytes = total_bytes.saturating_sub(done_bytes);
                let eta = if rate_bps > 0.0 {
                    let secs = remaining_bytes as f64 / rate_bps;
                    Some(Duration::from_secs_f64(secs))
                } else {
                    None
                };
                let iters = iter_count.swap(0, Ordering::Relaxed);
                let ex = exceeding_names.lock().unwrap().len();
                let stats_ex = stats_exceed.lock().unwrap().clone();
                let stats_ok = stats_ok.lock().unwrap().clone();
                println!(
                    "[STATS] {}/{} batches, {:.2} it/s, processed {}, rate {}/s, remaining {}, ETA {}, exceeding: {}",
                    done_batches,
                    total_batches,
                    iters as f64,
                    format_size(done_bytes, DECIMAL),
                    format_size(rate_bps as u64, DECIMAL),
                    format_size(remaining_bytes, DECIMAL),
                    eta.map(|d| format_duration(d).to_string())
                        .unwrap_or("--".to_string()),
                    ex
                );
                if stats_ex.count > 0 {
                    println!(
                        "  >exceed: min {}, max {}, mean {:.2}, stddev {:.2}, count {}",
                        stats_ex.min,
                        stats_ex.max,
                        stats_ex.mean(),
                        stats_ex.stddev(),
                        stats_ex.count
                    );
                }
                if stats_ok.count > 0 {
                    println!(
                        "  <=ok:    min {}, max {}, mean {:.2}, stddev {:.2}, count {}",
                        stats_ok.min,
                        stats_ok.max,
                        stats_ok.mean(),
                        stats_ok.stddev(),
                        stats_ok.count
                    );
                }
            }
        });
    }

    // Phase 2: process each entry batch, compute perceptual hashes and distances per group
    for (path_inside_zip, infos) in diff_entries {
        iter_count.fetch_add(1, Ordering::Relaxed);
        // info!("Processing perceptual hashes for {}", pi.display());
        // remember count before moving infos
        let infos_count = infos.len();
        let mut group_set = JoinSet::new();
        for info in infos {
            let path_inside_zip = path_inside_zip.clone();
            let path_to_zip = info.zip.clone();
            let size = info.size;
            group_set.spawn(async move {
                let f = Arc::new(RandomAccessFile::open(&path_to_zip)?);
                let arch = f.read_zip().await?;
                let ent = arch
                    .by_name(path_inside_zip.to_string_lossy().as_ref())
                    .ok_or(eyre!(
                        "Missing {} in {}",
                        path_inside_zip.display(),
                        path_to_zip.display()
                    ))?;
                let data = ent.bytes().await?;
                let img = load_from_memory(&data)?;
                let hash = image_hasher().hash_image(&img);
                Ok::<_, eyre::Report>((path_to_zip, hash, size))
            });
        }

        // Collect hashed results as they complete
        // Collect hashed results and accumulate processed bytes
        let mut results: Vec<(PathToZip, ImageHash)> = Vec::with_capacity(infos_count);
        while let Some(res) = group_set.join_next().await {
            // res yields (PathToZip, ImageHash, size)
            let (path_to_zip, image_hash, processed_bytes_inc) = res??;
            // accumulate processed bytes
            processed_bytes.fetch_add(processed_bytes_inc, Ordering::Relaxed);
            results.push((path_to_zip, image_hash));
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
            exceeding_names
                .lock()
                .unwrap()
                .insert(path_inside_zip.clone());
        }
        processed_batches.fetch_add(1, Ordering::Relaxed);
    }

    Ok(())
}
