use super::Progress;
use crate::size_of_thing::KnownSize;
use humantime::FormattedDuration;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use uom::si::f64::Information;

pub async fn track_progress<T, R, F, Fut>(
    items: impl IntoIterator<Item = T>,
    progress_display_interval: Duration,
    progress_task_queued_display_fn: impl Fn(&Progress) + Send + 'static,
    progress_task_received_display_fn: impl Fn(&Progress) + Send + 'static,
    progress_complete_display_fn: impl Fn(&Progress, FormattedDuration) + Send + 'static,
    mapping_fn: F,
    rate_limit: usize,
) -> eyre::Result<Vec<R>>
where
    T: KnownSize + Send + 'static,
    R: Send + 'static,
    F: Fn(T) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = eyre::Result<R>> + Send + 'static,
{
    let start = Instant::now();
    let items = items.into_iter().collect::<Vec<_>>();
    let mut progress = Progress::new(&items);
    let mut last_progress = Instant::now();

    // Spawn work
    let mut join_set: JoinSet<eyre::Result<Response<R>>> = JoinSet::new();
    let rate_limit: Option<Arc<Semaphore>> = match rate_limit {
        0 => None,
        x => Some(Arc::new(Semaphore::new(x))),
    };
    struct Response<R> {
        size: Information,
        user_data: R,
    }
    // Wrap mapping_fn in Arc to extend its lifetime for async tasks
    let mapping_fn = Arc::new(mapping_fn);
    for item in items {
        let rate_limit = rate_limit.clone();
        let processed_bytes = item.size_of();
        let mapping_fn = mapping_fn.clone();
        join_set.spawn(async move {
            let _permit = match rate_limit.as_ref() {
                Some(sem) => Some(sem.acquire().await),
                None => None,
            };
            let size = item.size_of();
            let user_data = mapping_fn(item).await?;
            Ok(Response { size, user_data })
        });

        // Track and log progress
        progress.track(1, processed_bytes);
        if Instant::now().duration_since(last_progress) >= progress_display_interval {
            progress_task_queued_display_fn(&progress);
            last_progress = Instant::now();
        }
    }

    // Complete work
    progress.reset();
    let mut rtn = Vec::new();
    while let Some(res) = join_set.join_next().await {
        let res = res??;
        rtn.push(res.user_data);
        progress.track(1, res.size);
        if Instant::now().duration_since(last_progress) >= progress_display_interval {
            progress_task_received_display_fn(&progress);
            last_progress = Instant::now();
        }
    }

    let elapsed =
        humantime::format_duration(Duration::from_millis(start.elapsed().as_millis() as u64));
    progress_complete_display_fn(&progress, elapsed);

    Ok(rtn)
}
