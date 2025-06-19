use crate::metrics::Metric;
use crate::progress::Progress;
use eyre::Result;

/// Metric for items processed per second
pub struct ItemsPerSecondMetric;

impl Metric for ItemsPerSecondMetric {
    fn title(&self) -> &'static str {
        "Item Throughput"
    }

    fn value(&self, progress: &Progress) -> Result<String> {
        let total_items: usize = progress.history.iter().map(|e| e.processed_items).sum();
        let elapsed = progress.start_time.elapsed().as_secs_f64().max(0.001);
        let rate = total_items as f64 / elapsed;
        Ok(format!("{rate:.1}"))
    }
}
