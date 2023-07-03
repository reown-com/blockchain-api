use tracing::info;

pub mod alloc;

pub async fn debug_metrics(alloc_metrics: alloc::AllocMetrics) {
    info!("Starting debug metrics collection");
    loop {
        info!("Collecting alloc metrics");
        alloc_metrics.collect_alloc_stats();
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    }
}
