use crate::metrics::Metric;
use crate::progress::Progress;
use eyre::Result;

/// Metric for total number of items
pub struct TotalItemsMetric;

impl Metric for TotalItemsMetric {
    fn title(&self) -> &'static str {
        "Total Items"
    }

    fn value(&self, progress: &Progress) -> Result<String> {
        Ok(progress.total_items.to_string())
    }
}
