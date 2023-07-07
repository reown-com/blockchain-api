use tracing::info;

pub mod alloc;
pub mod profiler;

// pub struct Config {
//     pub s3_bucket: Option<String>,
// }

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
