use crate::utils::error::{categorize_error, ErrorCategory};
use tokio::time::{sleep, Duration};

pub async fn with_retry<F, Fut, T>(
    name: &str,
    mut operation: F,
    max_retries: usize,
    initial_delay: Duration,
    max_delay: Duration,
) -> anyhow::Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<T>>,
{
    let mut delay = initial_delay;

    for i in 0..=max_retries {
        match operation().await {
            Ok(val) => return Ok(val),
            Err(e) if i < max_retries && categorize_error(&e) == ErrorCategory::Transient => {
                let jitter = Duration::from_millis(rand::random::<u64>() % 200);
                let current_delay = delay + jitter;

                log::warn!(
                    "[{}] Transient error (retry {}/{}): {}. Retrying in {:?}...",
                    name,
                    i + 1,
                    max_retries,
                    e,
                    current_delay
                );

                sleep(current_delay).await;

                delay = std::cmp::min(delay * 2, max_delay);
            }
            Err(e) => return Err(e),
        }
    }

    Err(anyhow::anyhow!("Retry limit exceeded for {}", name))
}