use std::future::Future;
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(25);

pub async fn wait_for<F, Fut, T>(timeout: Duration, label: &str, mut probe: F) -> T
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Option<T>>,
{
    let deadline = Instant::now() + timeout;

    loop {
        if let Some(value) = probe().await {
            return value;
        }

        if Instant::now() >= deadline {
            panic!("timed out after {timeout:?} waiting for {label}");
        }

        tokio::time::sleep(POLL_INTERVAL).await;
    }
}
