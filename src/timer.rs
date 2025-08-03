use std::time::Instant;
use tokio::time::Instant as AsyncInstant;

#[allow(unused)]
pub fn timed<T, F>(func: F) -> Result<T, anyhow::Error>
where
    F: FnOnce() -> Result<T, anyhow::Error>,
{
    let start = Instant::now();
    let result = func();
    let finish = Instant::now();

    let duration = finish.duration_since(start);
    tracing::info!(
        "completed in {} ns / {} ms",
        duration.as_nanos(),
        duration.as_millis()
    );

    tracing::Span::current()
        .record("duration", duration.as_millis())
        .record("duration_ns", duration.as_nanos());

    result
}

pub async fn timed_async<T, F>(func: F) -> Result<T, anyhow::Error>
where
    F: AsyncFnOnce() -> Result<T, anyhow::Error>,
{
    let start = AsyncInstant::now();
    let result = func().await;
    let finish = AsyncInstant::now();

    let duration = finish.duration_since(start);
    tracing::info!(
        "completed in {} ns / {} ms",
        duration.as_nanos(),
        duration.as_millis()
    );

    tracing::Span::current()
        .record("duration", duration.as_millis())
        .record("duration_ns", duration.as_nanos());

    result
}
