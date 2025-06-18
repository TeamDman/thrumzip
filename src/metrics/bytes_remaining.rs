use crate::metrics::Metric;
use crate::progress::Progress;
use eyre::Result;
use humansize::DECIMAL;
use humansize::format_size;
use uom::si::f64::Information;
use uom::si::information::byte;

/// Metric for bytes remaining
pub struct BytesRemainingMetric;

impl Metric for BytesRemainingMetric {
    fn title(&self) -> &'static str {
        "Bytes Remaining"
    }

    fn value(&self, progress: &Progress) -> Result<String> {
        // Total processed bytes
        let processed: Information = progress.history.iter().map(|e| e.processed_bytes).sum();
        // Remaining bytes
        let remaining: Information = progress.total_bytes - processed;
        // Format size in human-readable form
        let bytes: u64 = remaining.get::<byte>() as u64;
        Ok(format_size(bytes, DECIMAL))
    }
}
