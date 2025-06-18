use crate::metrics::Metric;
use crate::progress::Progress;
use chrono::Local;
use eyre::Result;

/// Metric for current local system time
pub struct CurrentTimeMetric;

impl Metric for CurrentTimeMetric {
    fn title(&self) -> &'static str {
        "Current Time"
    }

    fn value(&self, _progress: &Progress) -> Result<String> {
        Ok(Local::now().format("%Y-%m-%d %H:%M:%S").to_string())
    }
}
