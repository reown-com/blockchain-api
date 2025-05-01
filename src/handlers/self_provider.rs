use {
    super::SdkInfoParams,
    crate::{analytics::MessageSource, error::RpcError, handlers::RpcQueryParams, state::AppState},
    alloy::{
        providers::{Provider, ProviderBuilder},
        rpc::{
            client::RpcClient,
            json_rpc::{RequestPacket, ResponsePacket},
        },
        transports::{TransportError, TransportErrorKind, TransportFut},
    },
    bytes::Bytes,
    hyper::HeaderMap,
    relay_rpc::domain::ProjectId,
    std::{net::SocketAddr, sync::Arc, task::Poll},
    tower::Service,
};

#[derive(Clone)]
pub struct SelfProviderPool {
    pub state: Arc<AppState>,
    pub connect_info: SocketAddr,
    pub headers: HeaderMap,
    pub project_id: ProjectId,
    pub sdk_info: SdkInfoParams,
    pub session_id: Option<String>,
}

impl SelfProviderPool {
    pub fn get_provider(&self, chain_id: String, source: MessageSource) -> impl Provider {
        super::self_provider::provider(SelfRpcTransport {
            state: self.state.clone(),
            connect_info: self.connect_info,
            query: RpcQueryParams {
                chain_id,
                project_id: self.project_id.to_string(),
                provider_id: None,
                session_id: self.session_id.clone(),
                source: Some(source),
                sdk_info: self.sdk_info.clone(),
            },
            headers: self.headers.clone(),
        })
    }
}

pub fn provider(self_rpc_transport: SelfRpcTransport) -> impl Provider {
    ProviderBuilder::default().on_client(RpcClient::new(self_rpc_transport, false))
}

#[derive(thiserror::Error, Debug)]
pub enum SelfRpcTransportError {
    #[error("Request serialize: {0}")]
    RequestSerialize(serde_json::Error),

    #[error("RPC: {0}")]
    Rpc(RpcError),

    #[error("Response to_bytes: {0}")]
    ResponseToBytes(axum::Error),

    #[error("Response parse: {0}")]
    ResponseParse(serde_json::Error),
}

#[derive(Clone)]
pub struct SelfRpcTransport {
    pub state: Arc<AppState>,
    pub connect_info: SocketAddr,
    pub query: RpcQueryParams,
    pub headers: HeaderMap,
}

impl Service<RequestPacket> for SelfRpcTransport {
    type Error = TransportError;
    type Future = TransportFut<'static>;
    type Response = ResponsePacket;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: RequestPacket) -> Self::Future {
        let state = self.state.clone();
        let connect_info = self.connect_info;
        let query = self.query.clone();
        let headers = self.headers.clone();

        Box::pin(async move {
            let body = Bytes::copy_from_slice(
                req.serialize()
                    .map_err(|e| {
                        TransportErrorKind::custom(SelfRpcTransportError::RequestSerialize(e))
                    })?
                    .get()
                    .as_bytes(),
            );

            let result = crate::handlers::proxy::handler(
                axum::extract::State(state),
                axum::extract::ConnectInfo(connect_info),
                axum::extract::Query(query),
                headers,
                body,
            )
            .await
            .map_err(|e| TransportErrorKind::custom(SelfRpcTransportError::Rpc(e)))?;

            let bytes = hyper::body::to_bytes(result.into_body())
                .await
                .map_err(|e| {
                    TransportErrorKind::custom(SelfRpcTransportError::ResponseToBytes(e))
                })?;

            let response = serde_json::from_slice::<ResponsePacket>(bytes.as_ref())
                .map_err(|e| TransportErrorKind::custom(SelfRpcTransportError::ResponseParse(e)))?;

            Ok(response)
        })
    }
}
