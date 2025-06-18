use crate::metrics::Metric;
use crate::progress::Progress;
use eyre::Result;
use humantime::format_duration;
use std::time::Duration;
use uom::si::f64::Time;
use uom::si::time::millisecond;
use uom::si::time::second;

/// Metric for elapsed time since start
pub struct ElapsedTimeMetric;

impl Metric for ElapsedTimeMetric {
    fn title(&self) -> &'static str {
        "Elapsed Time"
    }

    fn value(&self, progress: &Progress) -> Result<String> {
        let elapsed = progress.start_time.elapsed();
        Ok(format_duration(Duration::from_millis(
            Time::new::<second>(elapsed.as_secs_f64()).get::<millisecond>() as u64,
        ))
        .to_string())
    }
}
