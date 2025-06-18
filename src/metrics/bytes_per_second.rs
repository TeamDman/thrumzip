use crate::metrics::Metric;
use crate::progress::Progress;
use crate::size_of_thing::KnownSize;
use eyre::Result;
use uom::si::f64::Information;
use uom::si::f64::InformationRate;
use uom::si::f64::Time;
use uom::si::time::second;

/// Metric for bytes processed per second
pub struct BytesPerSecondMetric;

impl Metric for BytesPerSecondMetric {
    fn title(&self) -> &'static str {
        "Data Throughput"
    }

    fn value(&self, progress: &Progress) -> Result<String> {
        // Total bytes processed in history
        let total_bytes: Information = progress.history.iter().map(|e| e.processed_bytes).sum();
        // Elapsed time since start
        let elapsed = Time::new::<second>(progress.start_time.elapsed().as_secs_f64().max(0.001));
        // Compute rate
        let rate: InformationRate = (total_bytes / elapsed).into();
        Ok(format!("{}/s", rate.human_size()))
    }
}
