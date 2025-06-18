use crate::metrics::Metric;
use crate::progress::Progress;
use crate::size_of_thing::KnownSize;
use eyre::Result;
use uom::si::f64::Information;

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
        Ok(remaining.human_size())
    }
}
