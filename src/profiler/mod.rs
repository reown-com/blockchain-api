#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ProfilerConfig {}

pub async fn run() {
    loop {
        if let Err(err) = wc::alloc::stats::update_jemalloc_metrics() {
            tracing::warn!(?err, "failed to collect jemalloc stats");
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    }
}
