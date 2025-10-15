use {
    super::{BridgingStatus, StorageBridgingItem, BRIDGING_TIMEOUT, STATUS_POLLING_INTERVAL},
    crate::{
        analytics::MessageSource,
        error::RpcError,
        state::AppState,
        storage::irn::OperationType,
        utils::crypto::get_erc20_balance,
    },
    alloy::primitives::U256,
    axum::{
        extract::{Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    ethers::types::H160 as EthersH160,
    std::{sync::Arc, time::SystemTime},
    tracing::error,
    wc::metrics::{future_metrics, FutureExt},
    yttrium::chain_abstraction::api::status::{
        StatusQueryParams,
        StatusResponse,
        StatusResponseCompleted,
        StatusResponseError,
        StatusResponsePendingObject,
    },
};

pub async fn handler(
    state: State<Arc<AppState>>,
    query_params: Query<StatusQueryParams>,
) -> Result<Response, RpcError> {
    handler_internal(state, query_params)
        .with_metrics(future_metrics!("handler_task", "name" => "ca_status"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    Query(query_params): Query<StatusQueryParams>,
) -> Result<Response, RpcError> {
    state
        .validate_project_access_and_quota(query_params.project_id.as_ref())
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
    let mut bridging_status_item = serde_json::from_slice::<StorageBridgingItem>(&irn_result)?;

    // Return without checking the balance if the status is completed or errored
    match bridging_status_item.status {
        BridgingStatus::Completed => {
            return Ok(Json(StatusResponse::Completed(StatusResponseCompleted {
                created_at: bridging_status_item.created_at,
            }))
            .into_response());
        }
        BridgingStatus::Error => {
            return Ok(Json(StatusResponse::Error(StatusResponseError {
                created_at: bridging_status_item.created_at,
                error: bridging_status_item.error_reason.unwrap_or_default(),
            }))
            .into_response());
        }
        _ => {}
    }

    // Check the balance of the wallet and the amount expected
    let wallet_balance = get_erc20_balance(
        &bridging_status_item.chain_id,
        EthersH160::from(<[u8; 20]>::from(bridging_status_item.contract)),
        EthersH160::from(<[u8; 20]>::from(bridging_status_item.wallet)),
        query_params.project_id.as_ref(),
        MessageSource::ChainAgnosticCheck,
        query_params.session_id.clone(),
    )
    .await?;

    if U256::from_be_bytes(wallet_balance.into()) >= bridging_status_item.amount_expected {
        // The balance was fullfilled, update the status to completed
        bridging_status_item.status = BridgingStatus::Completed;
        let irn_call_start = SystemTime::now();
        irn_client
            .set(
                query_params.orchestration_id,
                serde_json::to_vec(&bridging_status_item)?,
            )
            .await?;
        state
            .metrics
            .add_irn_latency(irn_call_start, OperationType::Set);

        return Ok(Json(StatusResponse::Completed(StatusResponseCompleted {
            created_at: bridging_status_item.created_at,
        }))
        .into_response());
    }

    // Check if the balance was not fullfilled with the right amount
    if U256::from_be_bytes(wallet_balance.into()) > bridging_status_item.amount_current {
        // We are not erroring here since there can be other transactions
        // that topped up the address, but log error for debugging purposes
        // to track if the bridging amount was less then expected
        error!(
            "Address was topped up with the amount less than expected: {} < {}",
            U256::from_be_bytes(wallet_balance.into()),
            bridging_status_item.amount_expected
        );
    }

    // Check if the timeout has been reached and update the item status to error
    if SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        > bridging_status_item.created_at + BRIDGING_TIMEOUT
    {
        bridging_status_item.status = BridgingStatus::Error;
        bridging_status_item.error_reason = Some("Bridging timeout".to_string());
        let irn_call_start = SystemTime::now();
        irn_client
            .set(
                query_params.orchestration_id,
                serde_json::to_vec(&bridging_status_item)?,
            )
            .await?;
        state
            .metrics
            .add_irn_latency(irn_call_start, OperationType::Set);

        return Ok(Json(StatusResponse::Error(StatusResponseError {
            created_at: bridging_status_item.created_at,
            error: bridging_status_item.error_reason.unwrap_or_default(),
        }))
        .into_response());
    }

    // The balance was not fullfilled return the pending status
    return Ok(Json(StatusResponse::Pending(StatusResponsePendingObject {
        created_at: bridging_status_item.created_at,
        check_in: STATUS_POLLING_INTERVAL,
    }))
    .into_response());
}
