pub mod worker;

use crate::metrics::BytesPerSecondMetric;
use crate::metrics::BytesProcessedMetric;
use crate::metrics::BytesRemainingMetric;
use crate::metrics::EstimatedTimeRemainingMetric;
use crate::metrics::ItemsPerSecondMetric;
use crate::metrics::ItemsProcessedMetric;
use crate::metrics::Metric;
use crate::metrics::PercentCompleteMetric;
use crate::metrics::RemainingItemsMetric;
use crate::metrics::TotalBytesMetric;
use crate::metrics::TotalItemsMetric;
use crate::size_of_thing::KnownSize;
use std::time::Instant;
use tracing::warn;
use uom::si::f64::Information;

pub struct Progress {
    pub total_items: usize,
    pub total_bytes: Information,
    pub start_time: Instant,
    pub history: Vec<ProgressHistoryEntry>,
}
pub struct ProgressHistoryEntry {
    pub processed_bytes: Information,
    pub processed_items: usize,
    pub timestamp: Instant,
}
impl Progress {
    pub fn new(items: impl IntoIterator<Item = impl KnownSize>) -> Self {
        let mut total_items = 0;
        let mut total_bytes = Information::new::<uom::si::information::byte>(0.0);
        for item in items {
            total_items += 1;
            total_bytes += item.size_of();
        }
        Self {
            start_time: Instant::now(),
            total_items,
            total_bytes,
            history: Vec::new(),
        }
    }
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.history.clear();
    }
    pub fn track(&mut self, processed_items: usize, processed_bytes: Information) {
        self.history.push(ProgressHistoryEntry {
            processed_bytes,
            processed_items,
            timestamp: Instant::now(),
        });
    }
}

impl std::fmt::Display for Progress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Render progress metrics using Metric trait implementations
        let result = (|| {
            write!(f, "({} complete) ", PercentCompleteMetric.value(self)?)?;
            write!(
                f,
                "[{}/{} items] ",
                ItemsProcessedMetric.value(self)?,
                TotalItemsMetric.value(self)?
            )?;
            write!(f, "{} items/s ", ItemsPerSecondMetric.value(self)?)?;
            write!(f, "({} remain), ", RemainingItemsMetric.value(self)?)?;
            write!(
                f,
                "[{}/{} processed] ",
                BytesProcessedMetric.value(self)?,
                TotalBytesMetric.value(self)?
            )?;
            write!(f, "{} ", BytesPerSecondMetric.value(self)?)?;
            write!(f, "({} remain), ", BytesRemainingMetric.value(self)?)?;
            write!(f, "ETA={}", EstimatedTimeRemainingMetric.value(self)?)?;
            eyre::Ok(())
        })();
        if let Err(e) = result {
            warn!("Error formatting progress metrics: {}", e);
            return Err(std::fmt::Error);
        }
        Ok(())
    }
}
