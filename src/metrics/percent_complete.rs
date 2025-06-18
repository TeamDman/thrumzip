use crate::metrics::Metric;
use crate::progress::Progress;
use eyre::Result;

/// Metric for percentage of items processed
pub struct PercentCompleteMetric;

impl Metric for PercentCompleteMetric {
    fn title(&self) -> &'static str {
        "Percent Complete"
    }

    fn value(&self, progress: &Progress) -> Result<String> {
        let processed: usize = progress.history.iter().map(|e| e.processed_items).sum();
        let total = progress.total_items;
        let percent = if total > 0 {
            processed as f64 / total as f64 * 100.0
        } else {
            0.0
        };
        Ok(format!("{:.1}%", percent))
    }
}
