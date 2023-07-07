use tracing::info;

pub mod alloc;
pub mod profiler;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DebugConfig {
    pub secret: String,
}

pub async fn debug_metrics(alloc_metrics: alloc::AllocMetrics, config: crate::env::Config) {
    info!("Initializing profiler upload context");
    profiler::init_upload_context(&config).await;

    info!("Starting debug metrics collection");
    loop {
        info!("Collecting alloc metrics");
        alloc_metrics.collect_alloc_stats();
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    }
}
