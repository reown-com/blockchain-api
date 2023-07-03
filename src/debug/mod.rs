pub mod alloc;

pub async fn debug_metrics(alloc_metrics: alloc::AllocMetrics) {
    loop {
        alloc_metrics.collect_alloc_stats();
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    }
}
