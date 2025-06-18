use humansize::DECIMAL;
use itertools::Itertools;
use std::time::Duration;
use std::time::Instant;
use uom::si::f64::Information;
use uom::si::f64::InformationRate;
use uom::si::f64::Time;
use uom::si::information::byte;
use uom::si::information_rate::byte_per_second;
use uom::si::time::millisecond;
use uom::si::time::second;

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
    pub fn new(total_items: usize, total_bytes: Information) -> Self {
        Self {
            start_time: Instant::now(),
            total_items,
            total_bytes,
            history: Vec::new(),
        }
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
        // How far back in time do we want to base our estimation on?
        // let window = Duration::from_secs(5); // 5 seconds
        let window = Duration::MAX; // Use all history for estimation

        let window_entries = self
            .history
            .iter()
            .rev()
            .take_while(|e| e.timestamp.elapsed() <= window)
            .collect_vec();
        let window_bytes_processed: Information =
            window_entries.iter().map(|e| e.processed_bytes).sum();
        let elapsed = Time::new::<second>(
            window_entries
                .iter()
                .map(|e| e.timestamp.elapsed())
                .max()
                .unwrap_or_default()
                .as_secs_f64(),
        );
        let data_rate: InformationRate = (window_bytes_processed / elapsed).into();
        let items_processed: usize = window_entries.iter().map(|e| e.processed_items).sum();
        let items_per_second = items_processed as f64 / elapsed.get::<second>().max(0.001);
        let remaining_bytes = self.total_bytes
            - self
                .history
                .iter()
                .map(|e| e.processed_bytes)
                .sum::<Information>();
        let remaining_time = Duration::from_secs(if data_rate.get::<byte_per_second>() > 0.0 {
            (remaining_bytes.get::<byte>() / data_rate.get::<byte_per_second>()) as u64
        } else {
            u64::MAX
        });

        let display_data_rate = format!(
            "{}/s",
            humansize::format_size_i(data_rate.get::<byte_per_second>(), DECIMAL)
        );
        let display_remaining_size =
            humansize::format_size(remaining_bytes.get::<byte>() as u64, DECIMAL);
        let display_eta = humantime::format_duration(remaining_time);
        let display_items_per_second = format!("{:.1}", items_per_second);

        let elapsed_time_str =
            humantime::format_duration(Duration::from_millis(elapsed.get::<millisecond>() as u64));

        let files_remaining_str = if self.total_items > 0 {
            format!(
                "{}/{}",
                self.total_items - items_processed,
                self.total_items
            )
        } else {
            String::from("0")
        };
        write!(
            f,
            "{elapsed_time_str} elapsed, {display_data_rate}, {display_remaining_size} remain, {display_eta} ETA, {display_items_per_second} files/s, {files_remaining_str} files remaining",
        )?;
        Ok(())
    }
}


