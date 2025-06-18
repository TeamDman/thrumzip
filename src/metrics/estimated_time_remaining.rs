use crate::metrics::Metric;
use crate::progress::Progress;
use eyre::Result;
use humantime::format_duration;
use std::time::Duration;
use uom::si::f64::Information;
use uom::si::f64::InformationRate;
use uom::si::f64::Time;
use uom::si::time::millisecond;
use uom::si::time::second;

/// Metric for estimated time remaining based on current data rate
pub struct EstimatedTimeRemainingMetric;

impl Metric for EstimatedTimeRemainingMetric {
    fn title(&self) -> &'static str {
        "Time Remaining"
    }

    fn value(&self, progress: &Progress) -> Result<String> {
        // Calculate total processed bytes
        let info_processed: Information = progress.history.iter().map(|e| e.processed_bytes).sum();
        // Compute elapsed time
        let time_elapsed =
            Time::new::<second>(progress.start_time.elapsed().as_secs_f64().max(0.001));
        // Compute data rate
        let info_rate: InformationRate = (info_processed / time_elapsed).into();
        // Compute remaining bytes
        let info_remaining = progress.total_bytes - info_processed;
        // Estimate remaining milliseconds
        let time_remaining =
            Duration::from_millis((info_remaining / info_rate).get::<millisecond>() as u64);
        Ok(format_duration(time_remaining).to_string())
    }
}
