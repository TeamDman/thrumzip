use crate::metrics::Metric;
use crate::progress::Progress;
use chrono::Local;
use eyre::Result;
use std::time::Duration;
use uom::si::f64::Information;
use uom::si::f64::InformationRate;
use uom::si::f64::Time;
use uom::si::information::byte;
use uom::si::information_rate::byte_per_second;
use uom::si::time::second;

/// Metric for estimated completion time (wall-clock time)
pub struct EstimatedCompletionTimeMetric;

impl Metric for EstimatedCompletionTimeMetric {
    fn title(&self) -> &'static str {
        "ETA Time"
    }

    fn value(&self, progress: &Progress) -> Result<String> {
        // Total processed bytes
        let processed: Information = progress.history.iter().map(|e| e.processed_bytes).sum();
        // Compute elapsed time
        let elapsed_sec = progress.start_time.elapsed().as_secs_f64().max(0.001);
        let elapsed = Time::new::<second>(elapsed_sec);
        // Compute rate
        let rate: InformationRate = (processed / elapsed).into();
        // Compute remaining bytes
        let remaining_bytes = progress.total_bytes - processed;
        // Estimate remaining duration
        let remaining_secs = if rate.get::<byte_per_second>() > 0.0 {
            (remaining_bytes.get::<byte>() / rate.get::<byte_per_second>()) as u64
        } else {
            u64::MAX
        };
        let dur = Duration::from_secs(remaining_secs);
        // Compute estimated completion DateTime
        let eta = Local::now() + chrono::Duration::from_std(dur)?;
        Ok(eta.format("%Y-%m-%d %H:%M:%S").to_string())
    }
}
