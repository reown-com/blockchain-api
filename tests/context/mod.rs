use async_trait::async_trait;
use test_context::AsyncTestContext;

use self::server::RpcProxy;

mod server;

pub struct ServerContext {
    pub server: RpcProxy,
}

#[async_trait]
impl AsyncTestContext for ServerContext {
    async fn setup() -> Self {
        let server = RpcProxy::start().await;
        Self { server }
    }

    async fn teardown(mut self) {
        self.server.shutdown().await;
    }
}

pub type TestResult<T> = Result<T, TestError>;

#[derive(Debug, thiserror::Error)]
pub enum TestError {
    #[error(transparent)]
    Elapsed(#[from] tokio::time::error::Elapsed),

    #[error(transparent)]
    RpcProxy(#[from] rpc_proxy::error::RpcError),
}
