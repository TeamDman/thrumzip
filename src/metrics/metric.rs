use crate::progress::Progress;
use eyre::Result;

/// Trait representing a progress metric to be displayed.
pub trait Metric {
    /// Returns the title of the metric (for display labeling).
    fn title(&self) -> &'static str;
    /// Computes the metric's value given the current progress state.
    fn value(&self, progress: &Progress) -> Result<String>;
}
