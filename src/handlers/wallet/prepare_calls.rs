use crate::analytics::MessageSource;
use crate::{handlers::HANDLER_TASK_METRICS, state::AppState};
use alloy::network::Ethereum;
use alloy::primitives::aliases::U192;
use alloy::primitives::{address, bytes, Address, Bytes, FixedBytes, U256, U64};
use alloy::providers::ReqwestProvider;
use alloy::sol_types::SolCall;
use alloy::sol_types::SolValue;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use url::Url;
use std::sync::Arc;
use thiserror::Error;
use tracing::error;
use wc::future::FutureExt;
use yttrium::erc7579::smart_sessions::ISmartSession::isSessionEnabledReturn;
use yttrium::erc7579::smart_sessions::{enableSessionSigCall, EnableSession, ISmartSession};
use yttrium::smart_accounts::account_address::AccountAddress;
use yttrium::{
    bundler::{config::BundlerConfig, pimlico::client::BundlerClient},
    chain::ChainId,
    entry_point::{EntryPointConfig, EntryPointVersion},
    smart_accounts::{nonce::get_nonce_with_key, safe::get_call_data},
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
    paymaster_service: Option<PaymasterService>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Permissions {
    context: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymasterService {
    url: Url,
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

    #[error("Permission context not long enough")]
    PermissionContextNotLongEnough,

    #[error("Permission context signature decompression error: {0}")]
    PermissionContextSignatureDecompression(fastlz_rs::DecompressError),

    #[error("Unsupported permission context mode: USE")]
    PermissionContextUnsupportedModeUse,

    #[error("Invalid permission context mode")]
    PermissionContextInvalidMode,

    #[error("Permission context ABI decode: {0}")]
    PermissionContextAbiDecode(alloy::sol_types::Error),

    #[error("Invalid permissionEnableSig for kernel account")]
    PermissionContextInvalidPermissionEnableSigForKernelAccount,

    #[error("Invalid permission context")]
    InvalidPermissionContext,

    #[error("Paymaster service capability is not supported")]
    PaymasterServiceUnsupported,

    #[error("Internal error")]
    InternalError(PrepareCallsInternalError),
}

#[derive(Error, Debug)]
pub enum PrepareCallsInternalError {
    #[error("Get nonce: {0}")]
    GetNonce(alloy::contract::Error),

    #[error("Estimate user operation gas price: {0}")]
    EstimateUserOperationGasPrice(eyre::Error),

    #[error("isSessionEnabled: {0}")]
    IsSessionEnabled(alloy::contract::Error),

    #[error("Compress session enabled: {0}")]
    CompressSessionEnabled(fastlz_rs::CompressError),
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

        if request.capabilities.paymaster_service.is_some() {
            return Err(PrepareCallsError::PaymasterServiceUnsupported);
        }

        // TODO check isSafe for request.from:
        // https://github.com/reown-com/web-examples/blob/32f9df464e2fa85ec49c21837d811cfe1437719e/advanced/wallets/react-wallet-v2/src/utils/UserOpBuilderUtil.ts#L39
        // What if it's not deployed yet?

        // TODO is7559Safe: https://github.com/reown-com/web-examples/blob/32f9df464e2fa85ec49c21837d811cfe1437719e/advanced/wallets/react-wallet-v2/src/lib/smart-accounts/builders/SafeUserOpBuilder.ts#L241
        // TODO shouldn't it always be 7579?

        // TODO get this from the Safe itself: https://github.com/reown-com/web-examples/blob/32f9df464e2fa85ec49c21837d811cfe1437719e/advanced/wallets/react-wallet-v2/src/lib/smart-accounts/builders/SafeUserOpBuilder.ts#L58
        // let safe_4337_module_address =

        // TODO get version from contract: https://github.com/reown-com/web-examples/blob/32f9df464e2fa85ec49c21837d811cfe1437719e/advanced/wallets/react-wallet-v2/src/lib/smart-accounts/builders/SafeUserOpBuilder.ts#L65

        let account_type = AccountType::Safe;

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

        // https://github.com/rhinestonewtf/module-sdk/blob/18ef7ca998c0d0a596572f18575e1b4967d9227b/src/module/smart-sessions/constants.ts#L2
        const SMART_SESSIONS_ADDRESS: Address =
            address!("82e5e20582d976f5db5e36c5a72c70d5711cef8b");

        let (validator_address, signature) = request
            .capabilities
            .permissions
            .context
            .split_at_checked(20)
            .ok_or(PrepareCallsError::PermissionContextNotLongEnough)?;

        let validator_address = Address::from_slice(validator_address);
        if validator_address != SMART_SESSIONS_ADDRESS {
            return Err(PrepareCallsError::InvalidPermissionContext);
        }

        let dummy_signature = {
            let DecodedSmartSessionSignature {
                permission_id,
                enable_session_data,
            } = decode_smart_session_signature(signature, account_type)?;

            const DUMMY_ECDSA_SIGNATURE: Bytes = bytes!("e8b94748580ca0b4993c9a1b86b5be851bfc076ff5ce3a1ff65bf16392acfcb800f9b4f1aef1555c7fce5599fffb17e7c635502154a0333ba21f3ae491839af51c");
            let signature = decode_signers(
                enable_session_data
                    .enable_session
                    .sessionToEnable
                    .sessionValidatorInitData
                    .clone(),
            )?
            .into_iter()
            .map(|t| match t {
                SignerType::Ecdsa => DUMMY_ECDSA_SIGNATURE,
                SignerType::Passkey => bytes!(""),
            })
            .collect::<Vec<_>>()
            .abi_encode();

            let smart_sessions = ISmartSession::new(SMART_SESSIONS_ADDRESS, provider.clone());
            let isSessionEnabledReturn {
                _0: session_enabled,
            } = smart_sessions
                .isSessionEnabled(permission_id, request.from.into())
                .call()
                .await
                .map_err(|e| {
                    PrepareCallsError::InternalError(PrepareCallsInternalError::IsSessionEnabled(e))
                })?;
            if session_enabled {
                let mut compress_state = fastlz_rs::CompressState::new();
                let compressed = Bytes::from(
                    compress_state
                        .compress_to_vec(&signature, fastlz_rs::CompressionLevel::Default)
                        .map_err(|e| {
                            PrepareCallsError::InternalError(
                                PrepareCallsInternalError::CompressSessionEnabled(e),
                            )
                        })?,
                );
                (Bytes::from([MODE_USE]), permission_id, compressed)
                    .abi_encode_packed()
                    .into()
            } else {
                let signature = (
                    enable_session_data.enable_session,
                    // EnableSession {
                    //     chainDigestIndex: enable_session_data.enable_session.chainDigestIndex,
                    //     hashesAndChainIds: enable_session_data.enable_session.hashesAndChainIds,
                    //     sessionToEnable: enable_session_data.enable_session.sessionToEnable,
                    //     permissionEnableSig: match account_type {
                    //         AccountType::Erc7579Implementation
                    //         | AccountType::Safe
                    //         | AccountType::Nexus => (
                    //             enable_session_data.validator,
                    //             enable_session_data.enable_session.permissionEnableSig,
                    //         )
                    //             .abi_encode_packed()
                    //             .into(),
                    //         AccountType::Kernel => (
                    //             [0x01],
                    //             enable_session_data.validator,
                    //             enable_session_data.enable_session.permissionEnableSig,
                    //         )
                    //             .abi_encode_packed()
                    //             .into(),
                    //     },
                    // },
                    signature,
                )
                    .abi_encode();

                let mut compress_state = fastlz_rs::CompressState::new();
                let compressed = Bytes::from(
                    compress_state
                        .compress_to_vec(&signature, fastlz_rs::CompressionLevel::Default)
                        .map_err(|e| {
                            PrepareCallsError::InternalError(
                                PrepareCallsInternalError::CompressSessionEnabled(e),
                            )
                        })?,
                );
                (Bytes::from([MODE_ENABLE]), permission_id, compressed)
                    .abi_encode_packed()
                    .into()
            }
        };

        // https://github.com/reown-com/web-examples/blob/32f9df464e2fa85ec49c21837d811cfe1437719e/advanced/wallets/react-wallet-v2/src/lib/smart-accounts/builders/SafeUserOpBuilder.ts#L110
        let nonce = get_nonce_with_key(
            &provider,
            request.from,
            &entry_point_config.address(),
            key_from_validator_address(validator_address),
        )
        .await
        .map_err(|e| PrepareCallsError::InternalError(PrepareCallsInternalError::GetNonce(e)))?;

        let pimlico_client = BundlerClient::new(BundlerConfig::new(
            format!(
                "https://rpc.walletconnect.com/v1/bundler?chainId={}&projectId={}&bundler=pimlico",
                chain_id.caip2_identifier(),
                state.config.server.testing_project_id.as_ref().unwrap(),
            )
            .parse()
            .unwrap(),
        ));
        let gas_price = pimlico_client
            .estimate_user_operation_gas_price()
            .await
            .map_err(|e| {
                PrepareCallsError::InternalError(
                    PrepareCallsInternalError::EstimateUserOperationGasPrice(e),
                )
            })?;

        let user_operation = UserOperationV07 {
            sender: request.from,
            nonce,
            factory: None,
            factory_data: None,
            call_data: get_call_data(request.calls),
            call_gas_limit: U256::from(2000000),
            verification_gas_limit: U256::from(2000000),
            pre_verification_gas: U256::from(2000000),
            max_fee_per_gas: gas_price.fast.max_fee_per_gas,
            max_priority_fee_per_gas: gas_price.fast.max_priority_fee_per_gas,
            paymaster: None,
            paymaster_verification_gas_limit: None,
            paymaster_post_op_gas_limit: None,
            paymaster_data: None,
            signature: dummy_signature,
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

// https://github.com/rhinestonewtf/module-sdk/blob/18ef7ca998c0d0a596572f18575e1b4967d9227b/src/account/types.ts#L4
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum AccountType {
    #[serde(rename = "erc7579-implementation")]
    Erc7579Implementation,

    #[serde(rename = "kernel")]
    Kernel,

    #[serde(rename = "safe")]
    Safe,

    #[serde(rename = "nexus")]
    Nexus,
}

struct EnableSessionData {
    enable_session: EnableSession,
    // validator: Address,
}

struct DecodedSmartSessionSignature {
    permission_id: FixedBytes<32>,
    enable_session_data: EnableSessionData,
}

// https://github.com/rhinestonewtf/module-sdk/blob/18ef7ca998c0d0a596572f18575e1b4967d9227b/src/module/smart-sessions/types.ts#L42
const MODE_USE: u8 = 0x00;
const MODE_ENABLE: u8 = 0x01;
const MODE_UNSAFE_ENABLE: u8 = 0x02;

// https://github.com/rhinestonewtf/module-sdk/blob/18ef7ca998c0d0a596572f18575e1b4967d9227b/src/module/smart-sessions/usage.ts#L209
fn decode_smart_session_signature(
    signature: &[u8],
    _account_type: AccountType,
) -> Result<DecodedSmartSessionSignature, PrepareCallsError> {
    let mode = signature
        .first()
        .ok_or(PrepareCallsError::PermissionContextNotLongEnough)?;
    let permission_id = signature
        .get(1..33)
        .ok_or(PrepareCallsError::PermissionContextNotLongEnough)?
        .try_into() // this error shouldn't happen
        .map_err(|_| PrepareCallsError::PermissionContextNotLongEnough)?;
    let compressed_data = signature
        .get(33..)
        .ok_or(PrepareCallsError::PermissionContextNotLongEnough)?;

    let data = fastlz_rs::decompress_to_vec(compressed_data, None)
        .map_err(PrepareCallsError::PermissionContextSignatureDecompression)?;

    match *mode {
        MODE_USE => {
            // https://github.com/rhinestonewtf/module-sdk/blob/18ef7ca998c0d0a596572f18575e1b4967d9227b/src/module/smart-sessions/usage.ts#L221
            // We aren't implementing this currently because it doesn't return the needed value (enableSessionData)
            Err(PrepareCallsError::PermissionContextUnsupportedModeUse)
        }
        MODE_ENABLE | MODE_UNSAFE_ENABLE => {
            let enableSessionSigCall {
                session: enable_session,
                signature: _,
            } = enableSessionSigCall::abi_decode_raw(&data, true)
                .map_err(PrepareCallsError::PermissionContextAbiDecode)?;
            // let is_kernel = account_type == AccountType::Kernel;
            // if is_kernel && enable_session.permissionEnableSig.starts_with(&[0x01]) {
            //     return Err(
            //         PrepareCallsError::PermissionContextInvalidPermissionEnableSigForKernelAccount,
            //     );
            // }

            // let permission_enable_sig =
            //     &enable_session.permissionEnableSig[if is_kernel { 1 } else { 0 }..];
            // let (validator, permission_enable_sig) = permission_enable_sig
            //     .split_at_checked(20)
            //     .ok_or(PrepareCallsError::PermissionContextNotLongEnough)?;
            // let validator = Address::from_slice(validator);

            Ok(DecodedSmartSessionSignature {
                permission_id,
                enable_session_data: EnableSessionData {
                    enable_session,
                    // enable_session: EnableSession {
                    //     chainDigestIndex: enable_session.chainDigestIndex,
                    //     hashesAndChainIds: enable_session.hashesAndChainIds,
                    //     sessionToEnable: enable_session.sessionToEnable,
                    //     permissionEnableSig: permission_enable_sig.into(), // TODO skip all this and just pass-through as-is
                    // },
                    // validator,
                },
            })
        }
        _ => Err(PrepareCallsError::PermissionContextInvalidMode),
    }
}

enum SignerType {
    Ecdsa,
    Passkey,
}

fn decode_signers(data: Bytes) -> Result<Vec<SignerType>, PrepareCallsError> {
    let mut data = data.into_iter();
    let signer_count = data
        .next()
        .ok_or(PrepareCallsError::InvalidPermissionContext)?; // TODO correct error variants
    let mut signers = Vec::with_capacity(signer_count as usize);
    for _i in 0..signer_count {
        let (signer_type, length) = match data.next() {
            Some(0) => (SignerType::Ecdsa, 20),
            Some(1) => (SignerType::Passkey, 64),
            _ => return Err(PrepareCallsError::InvalidPermissionContext), // TODO correct error variants
        };
        // ignore the actual signature
        for _i in 0..length {
            data.next()
                .ok_or(PrepareCallsError::InvalidPermissionContext)?; // TODO correct error variants
        }
        signers.push(signer_type);
    }
    if data.next().is_some() {
        return Err(PrepareCallsError::InvalidPermissionContext); // TODO correct error variants
    }
    Ok(signers)
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
}
