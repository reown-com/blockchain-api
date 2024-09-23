use crate::{error::new_error_response, handlers::HANDLER_TASK_METRICS, state::AppState};
use alloy_primitives::{Address, U256, U64};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use wc::future::FutureExt;
use yttrium::{
    chain::ChainId,
    entry_point::{EntryPointConfig, EntryPointVersion},
    smart_accounts::safe::{get_call_data, DUMMY_SIGNATURE},
    transaction::Transaction,
    user_operation::{user_operation_hash::UserOperationHash, UserOperationV07},
};

pub type PrepareCallsRequest = Vec<PrepareCallsRequestItem>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareCallsRequestItem {
    from: Address,
    chain_id: U64,
    calls: Vec<Transaction>,
    capabilities: Capabilities,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    permissions: Permissions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Permissions {
    context: String,
}

pub type PrepareCallsResponse = Vec<PrepareCallsResponseItem>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareCallsResponseItem {
    prepared_calls: PreparedCalls,
    signature_request: SignatureRequest,
    context: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparedCalls {
    r#type: SignatureRequestType,
    data: yttrium::user_operation::UserOperationV07,
    chain_id: U64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureRequestType {
    #[serde(rename = "user-operation-v07")]
    UserOpV7,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureRequest {
    hash: UserOperationHash,
}

#[derive(Error, Debug)]
pub enum PrepareCallsError {
    #[error("Invalid address")]
    InvalidAddress,

    #[error("Invalid chain ID")]
    InvalidChainId,
}

impl IntoResponse for PrepareCallsError {
    fn into_response(self) -> Response {
        match self {
            Self::InvalidAddress => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "from".to_owned(),
                    "Invalid address".to_owned(),
                )),
            )
                .into_response(),
            Self::InvalidChainId => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "chainId".to_owned(),
                    "Invalid chain ID".to_owned(),
                )),
            )
                .into_response(),
        }
    }
}

pub async fn handler(
    state: State<Arc<AppState>>,
    Json(request_payload): Json<PrepareCallsRequest>,
) -> Result<Response, PrepareCallsError> {
    handler_internal(state, request_payload)
        .with_metrics(HANDLER_TASK_METRICS.with_name("wallet_prepare_calls"))
        .await
}

#[tracing::instrument(skip(_state), level = "debug")]
async fn handler_internal(
    _state: State<Arc<AppState>>,
    request: PrepareCallsRequest,
) -> Result<Response, PrepareCallsError> {
    let mut response = Vec::with_capacity(request.len());
    for request in request {
        let chain_id = request.chain_id.to::<u64>();

        // TODO check isSafe for request.from:
        // https://github.com/reown-com/web-examples/blob/32f9df464e2fa85ec49c21837d811cfe1437719e/advanced/wallets/react-wallet-v2/src/utils/UserOpBuilderUtil.ts#L39
        // What if it's not deployed yet?

        // TODO is7559Safe: https://github.com/reown-com/web-examples/blob/32f9df464e2fa85ec49c21837d811cfe1437719e/advanced/wallets/react-wallet-v2/src/lib/smart-accounts/builders/SafeUserOpBuilder.ts#L241
        // TODO shouldn't it always be 7579?

        // TODO get this from the Safe itself: https://github.com/reown-com/web-examples/blob/32f9df464e2fa85ec49c21837d811cfe1437719e/advanced/wallets/react-wallet-v2/src/lib/smart-accounts/builders/SafeUserOpBuilder.ts#L58
        // let safe_4337_module_address =

        // TODO get version from contract: https://github.com/reown-com/web-examples/blob/32f9df464e2fa85ec49c21837d811cfe1437719e/advanced/wallets/react-wallet-v2/src/lib/smart-accounts/builders/SafeUserOpBuilder.ts#L65

        // TODO implement getNonce w/ permissionContext + key: https://github.com/reown-com/web-examples/blob/32f9df464e2fa85ec49c21837d811cfe1437719e/advanced/wallets/react-wallet-v2/src/lib/smart-accounts/builders/SafeUserOpBuilder.ts#L110

        // send_transaction
        let user_operation = UserOperationV07 {
            sender: request.from,
            nonce: U256::ZERO,
            factory: None,
            factory_data: None,
            call_data: get_call_data(request.calls),
            call_gas_limit: U256::ZERO,
            verification_gas_limit: U256::ZERO,
            pre_verification_gas: U256::ZERO,
            max_fee_per_gas: U256::ZERO,
            max_priority_fee_per_gas: U256::ZERO,
            paymaster: None,
            paymaster_verification_gas_limit: None,
            paymaster_post_op_gas_limit: None,
            paymaster_data: None,
            signature: DUMMY_SIGNATURE,
        };
        let hash = user_operation.hash(
            &EntryPointConfig {
                chain_id: ChainId::new_eip155(chain_id),
                version: EntryPointVersion::V07,
            }
            .address()
            .to_address(),
            chain_id,
        );

        response.push(PrepareCallsResponseItem {
            prepared_calls: PreparedCalls {
                r#type: SignatureRequestType::UserOpV7,
                data: user_operation,
                chain_id: request.chain_id,
            },
            signature_request: SignatureRequest { hash },
            context: request.capabilities.permissions.context,
        });
    }

    Ok(Json(response).into_response())
}
