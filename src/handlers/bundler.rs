use {
    super::HANDLER_TASK_METRICS,
    crate::{
        error::RpcError, providers::SupportedBundlerOps, state::AppState,
        utils::crypto::disassemble_caip2,
    },
    alloy::rpc::json_rpc::Id,
    axum::{
        extract::{Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    serde::{Deserialize, Serialize},
    std::sync::Arc,
    wc::future::FutureExt,
};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SupportedBundlers {
    Pimlico,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BundlerQueryParams {
    pub project_id: String,
    pub chain_id: String,
    pub bundler: SupportedBundlers,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BundlerJsonRpcRequest {
    pub id: Id,
    pub jsonrpc: Arc<str>,
    pub method: SupportedBundlerOps,
    pub params: serde_json::Value,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    query_params: Query<BundlerQueryParams>,
    Json(request_payload): Json<BundlerJsonRpcRequest>,
) -> Result<Response, RpcError> {
    handler_internal(state, query_params, request_payload)
        .with_metrics(HANDLER_TASK_METRICS.with_name("bundler_ops"))
        .await
}

#[tracing::instrument(skip_all, level = "debug")]
async fn handler_internal(
    State(state): State<Arc<AppState>>,
    Query(query_params): Query<BundlerQueryParams>,
    request_payload: BundlerJsonRpcRequest,
) -> Result<Response, RpcError> {
    state
        .validate_project_access_and_quota(&query_params.project_id.clone())
        .await?;
    let evm_chain_id = disassemble_caip2(&query_params.chain_id)?.1;
    let result = state
        .providers
        .bundler_ops_provider
        .bundler_rpc_call(
            &evm_chain_id,
            request_payload.id,
            request_payload.jsonrpc,
            &request_payload.method,
            request_payload.params,
        )
        .await?;

    Ok(Json(result).into_response())
}
