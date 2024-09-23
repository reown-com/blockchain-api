use crate::analytics::MessageSource;
use crate::{handlers::HANDLER_TASK_METRICS, state::AppState};
use alloy::network::Ethereum;
use alloy::primitives::aliases::U192;
use alloy::primitives::{address, Address, Bytes, U256, U64};
use alloy::providers::ReqwestProvider;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tracing::error;
use wc::future::FutureExt;
use yttrium::smart_accounts::account_address::AccountAddress;
use yttrium::{
    chain::ChainId,
    entry_point::{EntryPointConfig, EntryPointVersion},
    smart_accounts::{
        nonce::get_nonce_with_key,
        safe::{get_call_data, DUMMY_SIGNATURE},
    },
    transaction::Transaction,
    user_operation::{user_operation_hash::UserOperationHash, UserOperationV07},
};

pub type PrepareCallsRequest = Vec<PrepareCallsRequestItem>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareCallsRequestItem {
    from: AccountAddress,
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
    context: Bytes,
}

pub type PrepareCallsResponse = Vec<PrepareCallsResponseItem>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareCallsResponseItem {
    prepared_calls: PreparedCalls,
    signature_request: SignatureRequest,
    context: Bytes,
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

    #[error("Permission context doesn't start with address")]
    PermissionContextDoesntStartWithAddress,

    #[error("Invalid permission context")]
    InvalidPermissionContext,

    #[error("Internal error")]
    InternalError(PrepareCallsInternalError),
}

#[derive(Error, Debug)]
pub enum PrepareCallsInternalError {
    #[error("Get nonce: {0}")]
    GetNonce(alloy::contract::Error),
}

impl IntoResponse for PrepareCallsError {
    fn into_response(self) -> Response {
        match self {
            Self::InternalError(e) => {
                error!("HTTP server error: (prepare_calls) {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            e => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": e.to_string(),
                })),
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

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    request: PrepareCallsRequest,
) -> Result<Response, PrepareCallsError> {
    let mut response = Vec::with_capacity(request.len());
    for request in request {
        let chain_id = ChainId::new_eip155(request.chain_id.to::<u64>());

        // TODO check isSafe for request.from:
        // https://github.com/reown-com/web-examples/blob/32f9df464e2fa85ec49c21837d811cfe1437719e/advanced/wallets/react-wallet-v2/src/utils/UserOpBuilderUtil.ts#L39
        // What if it's not deployed yet?

        // TODO is7559Safe: https://github.com/reown-com/web-examples/blob/32f9df464e2fa85ec49c21837d811cfe1437719e/advanced/wallets/react-wallet-v2/src/lib/smart-accounts/builders/SafeUserOpBuilder.ts#L241
        // TODO shouldn't it always be 7579?

        // TODO get this from the Safe itself: https://github.com/reown-com/web-examples/blob/32f9df464e2fa85ec49c21837d811cfe1437719e/advanced/wallets/react-wallet-v2/src/lib/smart-accounts/builders/SafeUserOpBuilder.ts#L58
        // let safe_4337_module_address =

        // TODO get version from contract: https://github.com/reown-com/web-examples/blob/32f9df464e2fa85ec49c21837d811cfe1437719e/advanced/wallets/react-wallet-v2/src/lib/smart-accounts/builders/SafeUserOpBuilder.ts#L65

        let entry_point_config = EntryPointConfig {
            chain_id,
            version: EntryPointVersion::V07,
        };

        let provider = ReqwestProvider::<Ethereum>::new_http(
            format!(
                "https://rpc.walletconnect.com/v1?chainId={}&projectId={}&source={}",
                chain_id.caip2_identifier(),
                state.config.server.testing_project_id.as_ref().unwrap(),
                MessageSource::WalletPrepareCalls,
            )
            .parse()
            .unwrap(),
        );

        let nonce = {
            // https://github.com/rhinestonewtf/module-sdk/blob/18ef7ca998c0d0a596572f18575e1b4967d9227b/src/module/smart-sessions/constants.ts#L2
            const SMART_SESSIONS_ADDRESS: Address =
                address!("82e5e20582d976f5db5e36c5a72c70d5711cef8b");

            let validator_address = validator_address_from_permission_context(
                &request.capabilities.permissions.context,
            )?;
            if validator_address != SMART_SESSIONS_ADDRESS {
                return Err(PrepareCallsError::InvalidPermissionContext);
            }

            // https://github.com/reown-com/web-examples/blob/32f9df464e2fa85ec49c21837d811cfe1437719e/advanced/wallets/react-wallet-v2/src/lib/smart-accounts/builders/SafeUserOpBuilder.ts#L110
            let key = key_from_validator_address(validator_address);
            get_nonce_with_key(&provider, request.from, &entry_point_config.address(), key)
                .await
                .map_err(|e| {
                    PrepareCallsError::InternalError(PrepareCallsInternalError::GetNonce(e))
                })?
        };

        // TODO prepare paymaster info
        // TODO get special dummy signature: https://github.com/reown-com/web-examples/blob/32f9df464e2fa85ec49c21837d811cfe1437719e/advanced/wallets/react-wallet-v2/src/lib/smart-accounts/builders/ContextBuilderUtil.ts#L190

        let user_operation = UserOperationV07 {
            sender: request.from,
            nonce,
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
            &entry_point_config.address().to_address(),
            chain_id.eip155_chain_id(),
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

fn key_from_validator_address(validator_address: Address) -> U192 {
    U192::from_be_bytes({
        let mut key = [0u8; 24];
        key[..20].copy_from_slice(&validator_address.into_array());
        key
    })
}

fn validator_address_from_permission_context(
    context: &Bytes,
) -> Result<Address, PrepareCallsError> {
    Ok(Address::from_slice(context.get(0..20).ok_or(
        PrepareCallsError::PermissionContextDoesntStartWithAddress,
    )?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::bytes;

    #[test]
    fn test_key_from_validator_address() {
        let validator_address = address!("abababababababababababababababababababab");
        let key = key_from_validator_address(validator_address);
        assert_eq!(
            key.to_be_bytes_vec(),
            bytes!("abababababababababababababababababababab00000000").to_vec()
        );
    }

    #[test]
    fn test_validator_address_from_permission_context() {
        assert_eq!(
            validator_address_from_permission_context(&bytes!(
                "abababababababababababababababababababab"
            ))
            .unwrap(),
            address!("abababababababababababababababababababab")
        );
        assert_eq!(
            validator_address_from_permission_context(&bytes!(
                "ababababababababababababababababababababff"
            ))
            .unwrap(),
            address!("abababababababababababababababababababab")
        );
        assert!(validator_address_from_permission_context(&bytes!(
            "ababababababababababababababababababab"
        ))
        .is_err());
    }
}
