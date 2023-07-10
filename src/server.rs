use {
    std::{future::Future, time::Duration},
    wc::{
        future::StaticFutureExt,
        metrics::{counter, otel},
    },
};

/// Global `hyper` service task executor that uses the `tokio` runtime and adds
/// metrics for the executed tasks.
#[derive(Clone)]
pub struct ServiceTaskExecutor {
    /// Apply a timeout to all service tasks to prevent them from becoming
    /// zombies for various reasons.
    timeout: Duration,
}

impl ServiceTaskExecutor {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl<F> hyper::rt::Executor<F> for ServiceTaskExecutor
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        let timeout = self.timeout;

        async move {
            // Number of hyper service tasks started.
            counter!("service_task_started", 1);

            let completed = tokio::time::timeout(timeout, fut).await.is_ok();

            // Number of hyper service tasks completed.
            counter!("service_task_finished", 1, &[otel::KeyValue::new(
                "completed",
                completed
            )]);
        }
        .spawn("server::ServiceTaskExecutor::execute");
    }
}
