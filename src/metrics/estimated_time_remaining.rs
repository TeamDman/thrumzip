use crate::metrics::Metric;
use crate::progress::Progress;
use eyre::Result;
use humantime::format_duration;
use std::time::Duration;
use uom::si::f64::Information;
use uom::si::f64::InformationRate;
use uom::si::f64::Time;
use uom::si::information::byte;
use uom::si::information_rate::byte_per_second;
use uom::si::time::millisecond;
use uom::si::time::second;

/// Metric for estimated time remaining based on current data rate
pub struct EstimatedTimeRemainingMetric;

impl Metric for EstimatedTimeRemainingMetric {
    fn title(&self) -> &'static str {
        "Time Remaining"
    }

    fn value(&self, progress: &Progress) -> Result<String> {
        let mut skipped_entry_count = 0;
        let mut skipped_bytes = Information::new::<byte>(0.0);
        let skip_threshold = Duration::from_millis(1200);
        for skipped_entry in progress
            .history
            .iter()
            .filter(|e| (e.timestamp - progress.start_time) < skip_threshold)
        {
            skipped_entry_count += 1;
            skipped_bytes += skipped_entry.processed_bytes;
        }

        let skipped_time = progress
            .history
            .iter()
            .skip(skipped_entry_count)
            .map(|e| e.timestamp - progress.start_time)
            .sum::<Duration>();

        // Calculate total processed bytes (excluding skipped)
        let processed_bytes: Information = progress
            .history
            .iter()
            .skip(skipped_entry_count)
            .map(|e| e.processed_bytes)
            .sum();

        // Compute elapsed time, excluding before the start threshold
        let elapsed_time = Time::new::<second>(
            progress
                .start_time
                .elapsed()
                .checked_sub(skipped_time)
                .unwrap_or_default()
                .as_secs_f64()
                .max(0.001),
        );

        // If no progress, can't estimate
        if processed_bytes.value == 0.0 {
            return Ok("unknown".to_string());
        }

        // Compute data rate
        let throughput_bytes: InformationRate = (processed_bytes / elapsed_time).into();
        let min_throughput = InformationRate::new::<byte_per_second>(1e-6); // avoid div by zero
        let throughput = throughput_bytes.max(min_throughput);

        // Compute remaining bytes (exclude skipped only once)
        let remaining_bytes = (progress.total_bytes - skipped_bytes - processed_bytes).max(Information::new::<byte>(0.0));

        // If nothing remains, return 0s
        if remaining_bytes.value <= 0.0 {
            return Ok("0s".to_string());
        }

        // Estimate remaining milliseconds
        let remaining_time = Duration::from_millis(
            (remaining_bytes / throughput)
                .get::<millisecond>() as u64,
        );
        Ok(format_duration(remaining_time).to_string())
    }
}
