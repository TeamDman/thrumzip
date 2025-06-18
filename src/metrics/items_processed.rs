use crate::metrics::Metric;
use crate::progress::Progress;
use eyre::Result;

/// Metric for total items processed
pub struct ItemsProcessedMetric;

impl Metric for ItemsProcessedMetric {
    fn title(&self) -> &'static str {
        "Items Processed"
    }

    fn value(&self, progress: &Progress) -> Result<String> {
        let processed: usize = progress.history.iter().map(|e| e.processed_items).sum();
        Ok(processed.to_string())
    }
}
