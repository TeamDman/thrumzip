use crate::metrics::Metric;
use crate::progress::Progress;
use eyre::Result;
use humansize::{format_size, DECIMAL};
use uom::si::f64::Information;
use uom::si::information::byte;

/// Metric for total bytes to process
pub struct TotalBytesMetric;

impl Metric for TotalBytesMetric {
    fn title(&self) -> &'static str {
        "Total Bytes"
    }

    fn value(&self, progress: &Progress) -> Result<String> {
        let total: Information = progress.total_bytes;
        let bytes: u64 = total.get::<byte>() as u64;
        Ok(format_size(bytes, DECIMAL))
    }
}
