use crate::metrics::Metric;
use crate::progress::Progress;
use eyre::Result;

/// Metric for number of items remaining
pub struct RemainingItemsMetric;

impl Metric for RemainingItemsMetric {
    fn title(&self) -> &'static str {
        "Items Remaining"
    }

    fn value(&self, progress: &Progress) -> Result<String> {
        let processed: usize = progress.history.iter().map(|e| e.processed_items).sum();
        let remaining = progress.total_items - processed;
        Ok(remaining.to_string())
    }
}
