use {
    super::{super::HANDLER_TASK_METRICS, BridgingStatus, StorageBridgingItem},
    crate::{
        analytics::MessageSource, error::RpcError, state::AppState, storage::irn::OperationType,
        utils::crypto::get_balance,
    },
    alloy::primitives::U256,
    axum::{
        extract::{Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    ethers::types::H160 as EthersH160,
    serde::{Deserialize, Serialize},
    std::{sync::Arc, time::SystemTime},
    wc::future::FutureExt,
};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    pub project_id: String,
    pub orchestration_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    status: BridgingStatus,
    created_at: usize,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    query_params: Query<QueryParams>,
) -> Result<Response, RpcError> {
    handler_internal(state, query_params)
        .with_metrics(HANDLER_TASK_METRICS.with_name("ca_status"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    Query(query_params): Query<QueryParams>,
) -> Result<Response, RpcError> {
    state
        .validate_project_access_and_quota(&query_params.project_id.clone())
        .await?;

    let irn_client = state.irn.as_ref().ok_or(RpcError::IrnNotConfigured)?;

    // Get the bridging request status from the IRN
    let irn_call_start = SystemTime::now();
    let irn_result = irn_client
        .get(query_params.orchestration_id.clone())
        .await?
        .ok_or(RpcError::OrchestrationIdNotFound(
            query_params.orchestration_id.clone(),
        ))?;
    state
        .metrics
        .add_irn_latency(irn_call_start, OperationType::Get);
    let mut bridging_status_item = serde_json::from_str::<StorageBridgingItem>(&irn_result)?;

    // Return without checking the balance if the status is completed or errored
    if bridging_status_item.status == BridgingStatus::Completed
        || bridging_status_item.status == BridgingStatus::Error
    {
        return Ok(Json(StatusResponse {
            status: bridging_status_item.status,
            created_at: bridging_status_item.created_at,
        })
        .into_response());
    }

    // Check the balance of the wallet and the amount expected
    let wallet_balance = get_balance(
        &bridging_status_item.chain_id,
        EthersH160::from(<[u8; 20]>::from(bridging_status_item.wallet)),
        &query_params.project_id,
        MessageSource::ChainAgnosticCheck,
    )
    .await?;
    if U256::from_be_bytes(wallet_balance.into()) < bridging_status_item.amount_expected {
        // The balance was not fullfilled return the same pending status
        return Ok(Json(StatusResponse {
            status: bridging_status_item.status,
            created_at: bridging_status_item.created_at,
        })
        .into_response());
    } else {
        // The balance was fullfilled, update the status to completed
        bridging_status_item.status = BridgingStatus::Completed;
        let irn_call_start = SystemTime::now();
        irn_client
            .set(
                query_params.orchestration_id,
                serde_json::to_string(&bridging_status_item)?.into(),
            )
            .await?;
        state
            .metrics
            .add_irn_latency(irn_call_start, OperationType::Set);
    }

    return Ok(Json(StatusResponse {
        status: bridging_status_item.status,
        created_at: bridging_status_item.created_at,
    })
    .into_response());
}
