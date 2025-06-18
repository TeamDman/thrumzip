use crate::metrics::Metric;
use crate::progress::Progress;
use eyre::Result;
use humansize::{format_size, DECIMAL};
use uom::si::f64::Information;
use uom::si::information::byte;

/// Metric for total bytes processed
pub struct BytesProcessedMetric;

impl Metric for BytesProcessedMetric {
    fn title(&self) -> &'static str {
        "Bytes Processed"
    }

    fn value(&self, progress: &Progress) -> Result<String> {
        let processed: Information = progress
            .history
            .iter()
            .map(|e| e.processed_bytes)
            .sum();
        let bytes: u64 = processed.get::<byte>() as u64;
        Ok(format_size(bytes, DECIMAL))
    }
}
