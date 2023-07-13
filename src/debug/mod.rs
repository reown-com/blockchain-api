#[derive(Debug, Clone, serde::Deserialize)]
pub struct DebugConfig {
    pub secret: String,
}

pub async fn debug_metrics() {
    loop {
        if let Err(err) = wc::alloc::stats::update_jemalloc_metrics() {
            tracing::warn!(?err, "failed to collect jemalloc stats");
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    }
}
