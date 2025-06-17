// Toy demonstration: UOM + humantime + humansize for ETA/progress
use humansize::DECIMAL;
use humansize::format_size;
use humantime::format_duration;
use rand::Rng;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use uom::si::f64::Information;
use uom::si::information::byte;

fn main() {
    // Config
    const NUM_BATCHES: usize = 5000;
    const MIN_FILES: usize = 2;
    const MAX_FILES: usize = 7;
    const MIN_SIZE: u64 = 1_000; // 1 KB
    const MAX_SIZE: u64 = 5_000_000; // 5 MB
    const TIME_PER_BYTE: f64 = 0.000_000_02;

    // Generate batches
    let mut rng = rand::thread_rng();
    let mut batches = Vec::with_capacity(NUM_BATCHES);
    let mut total_bytes = Information::new::<byte>(0.0);
    for _ in 0..NUM_BATCHES {
        let num_files = rng.gen_range(MIN_FILES..=MAX_FILES);
        let mut files = Vec::with_capacity(num_files);
        for _ in 0..num_files {
            let sz = rng.gen_range(MIN_SIZE..=MAX_SIZE);
            files.push(Information::new::<byte>(sz as f64));
            total_bytes += Information::new::<byte>(sz as f64);
        }
        batches.push(files);
    }

    // Progress state
    let mut processed_batches = 0;
    let mut processed_files = 0;
    let mut processed_bytes = Information::new::<byte>(0.0);
    let mut last_print = Instant::now();
    let start = Instant::now();
    let mut files_this_sec = 0;
    let mut bytes_this_sec = 0.0;

    for files in batches.iter() {
        for file in files {
            // Simulate processing time proportional to file size
            let sleep_time = Duration::from_secs_f64(file.get::<byte>() * TIME_PER_BYTE);
            sleep(sleep_time);
            processed_files += 1;
            processed_bytes += *file;
            files_this_sec += 1;
            bytes_this_sec += file.get::<byte>();
        }
        processed_batches += 1;
        // Print every ~1s
        if last_print.elapsed() >= Duration::from_secs(1) || processed_batches == NUM_BATCHES {
            let elapsed = start.elapsed();
            let bps = bytes_this_sec / last_print.elapsed().as_secs_f64().max(0.001);
            let fps = files_this_sec as f64 / last_print.elapsed().as_secs_f64().max(0.001);
            let remaining_bytes = total_bytes - processed_bytes;
            let rate = processed_bytes.get::<byte>() / elapsed.as_secs_f64().max(0.001);
            let eta_secs = if rate > 0.0 {
                remaining_bytes.get::<byte>() / rate
            } else {
                0.0
            };
            println!(
                "Batch {}/{} | Files/sec: {:.1} | Bytes/sec: {}/s | Remaining: {} | ETA: {}",
                processed_batches,
                NUM_BATCHES,
                fps,
                format_size(bps as u64, DECIMAL),
                format_size(remaining_bytes.get::<byte>() as u64, DECIMAL),
                format_duration(Duration::from_secs_f64(eta_secs))
            );
            last_print = Instant::now();
            files_this_sec = 0;
            bytes_this_sec = 0.0;
        }
    }
    println!(
        "Done! Processed {} batches, {} files, {} total.",
        processed_batches,
        processed_files,
        format_size(processed_bytes.get::<byte>() as u64, DECIMAL)
    );
}
