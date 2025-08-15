use {
    super::{
        call_id::{CallId, CallIdInner},
        prepare_calls::{
            decode_smart_session_signature, encode_use_or_enable_smart_session_signature,
            split_permissions_context_and_check_validator, AccountType,
            DecodedSmartSessionSignature, PrepareCallsError,
        },
        types::PreparedCalls,
    },
    crate::{
        analytics::MessageSource,
        handlers::{
            sessions::{
                cosign::{self, CoSignQueryParams},
                get::{
                    get_session_context, GetSessionContextError, InternalGetSessionContextError,
                },
                CoSignRequest,
            },
            HANDLER_TASK_METRICS,
        },
        state::AppState,
        utils::{crypto::UserOperation, simple_request_json::SimpleRequestJson},
    },
    alloy::{
        primitives::{Bytes, U64},
        providers::ProviderBuilder,
    },
    axum::{
        extract::{Path, Query, State},
        response::IntoResponse,
    },
    // http_body_util::BodyExt,
    parquet::data_type::AsBytes,
    serde::{Deserialize, Serialize},
    std::sync::Arc,
    thiserror::Error,
    uuid::Uuid,
    wc::future::FutureExt,
    yttrium::{
        bundler::{client::BundlerClient, config::BundlerConfig},
        chain::ChainId,
        entry_point::{EntryPointConfig, EntryPointVersion},
        erc7579::smart_sessions::SmartSessionMode,
        user_operation::UserOperationV07,
    },
};

pub type SendPreparedCallsRequest = Vec<SendPreparedCallsRequestItem>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendPreparedCallsRequestItem {
    prepared_calls: PreparedCalls,
    signature: Bytes,
    context: Uuid,
}

pub type SendPreparedCallsResponse = Vec<CallId>;

#[derive(Error, Debug)]
pub enum SendPreparedCallsError {
    #[error("Invalid address")]
    InvalidAddress,

    #[error("Invalid chain ID")]
    InvalidChainId,
    #[error("Cosign error: {0}")]
    Cosign(String),

    #[error("Permission not found")]
    PermissionNotFound,

    #[error("PCI not found")]
    PciNotFound,

    #[error("Permission context not long enough")]
    PermissionContextNotLongEnough,

    #[error("Unsupported permission context mode: USE")]
    PermissionContextUnsupportedModeUse,

    #[error("Invalid permission context mode")]
    PermissionContextInvalidMode,

    #[error("Permission context ABI decode: {0}")]
    PermissionContextAbiDecode(alloy::sol_types::Error),

    #[error("Invalid permissionEnableSig for kernel account")]
    PermissionContextInvalidPermissionEnableSigForKernelAccount,

    // TODO refactor these errors to not depend on the other handler
    #[error("SplitPermissionsContextAndCheckValidator: {0}")]
    SplitPermissionsContextAndCheckValidator(PrepareCallsError),

    #[error("DecodeSmartSessionSignature: {0}")]
    DecodeSmartSessionSignature(PrepareCallsError),

    #[error("EncodeUseOrEnableSmartSessionSignature: {0}")]
    EncodeUseOrEnableSmartSessionSignature(PrepareCallsError),

    #[error("Invalid permission context")]
    InvalidPermissionContext,

    #[error("Paymaster service capability is not supported")]
    PaymasterServiceUnsupported,

    #[error("eth_sendUserOperation: {0}")]
    SendUserOperation(eyre::Report),

    #[error("Internal error")]
    InternalError(SendPreparedCallsInternalError),
}

#[derive(Error, Debug)]
pub enum SendPreparedCallsInternalError {
    #[error("IRN not configured")]
    IrnNotConfigured,

    #[error("Cosign: {0}")]
    Cosign(String),

    #[error("Cosign unsuccessful: {0:?}")]
    CosignUnsuccessful(std::result::Result<bytes::Bytes, axum::Error>),

    #[error("Cosign read response: {0}")]
    CosignReadResponse(axum::Error),

    #[error("Cosign parse response: {0}")]
    CosignParseResponse(serde_json::Error),

    #[error("Cosign response missing signature")]
    CosignResponseMissingSignature,

    #[error("Cosign response signature not string")]
    CosignResponseSignatureNotString,

    #[error("Cosign response signature not hex: {0}")]
    CosignResponseSignatureNotHex(hex::FromHexError),

    #[error("Get session context: {0}")]
    GetSessionContextError(InternalGetSessionContextError),

    #[error("isSessionEnabled: {0}")]
    IsSessionEnabled(alloy::contract::Error),
}

impl SendPreparedCallsError {
    pub fn is_internal(&self) -> bool {
        matches!(self, SendPreparedCallsError::InternalError(_))
    }
}

pub async fn handler(
    state: State<Arc<AppState>>,
    project_id: String,
    request: SendPreparedCallsRequest,
) -> Result<SendPreparedCallsResponse, SendPreparedCallsError> {
    handler_internal(state, project_id, request)
        .with_metrics(HANDLER_TASK_METRICS.with_name("wallet_send_prepared_calls"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    project_id: String,
    request: SendPreparedCallsRequest,
) -> Result<SendPreparedCallsResponse, SendPreparedCallsError> {
    let mut response = Vec::with_capacity(request.len());
    for request in request {
        let chain_id = ChainId::new_eip155(request.prepared_calls.chain_id.to::<u64>());
        let cosign_signature =
            {
                let cosign_request =
                    CoSignRequest {
                        pci: request.context.to_string(),
                        user_op: UserOperation {
                            sender: ethers::types::H160::from_slice(
                                request.prepared_calls.data.sender.to_address().as_bytes(),
                            ),
                            nonce: ethers::types::U256::from(
                                &request.prepared_calls.data.nonce.to_be_bytes(),
                            ),
                            call_data: ethers::types::Bytes::from(
                                request.prepared_calls.data.call_data.to_vec(),
                            ),
                            call_gas_limit: ethers::types::U128::from(
                                &request
                                    .prepared_calls
                                    .data
                                    .call_gas_limit
                                    .to_be_bytes::<32>()[16..],
                            ),
                            verification_gas_limit: ethers::types::U128::from(
                                &request
                                    .prepared_calls
                                    .data
                                    .verification_gas_limit
                                    .to_be_bytes::<32>()[16..],
                            ),
                            pre_verification_gas: ethers::types::U256::from(
                                &request
                                    .prepared_calls
                                    .data
                                    .pre_verification_gas
                                    .to_be_bytes(),
                            ),
                            max_priority_fee_per_gas: ethers::types::U128::from(
                                &request
                                    .prepared_calls
                                    .data
                                    .max_priority_fee_per_gas
                                    .to_be_bytes::<32>()[16..],
                            ),
                            max_fee_per_gas: ethers::types::U128::from(
                                &request
                                    .prepared_calls
                                    .data
                                    .max_fee_per_gas
                                    .to_be_bytes::<32>()[16..],
                            ),
                            signature: ethers::types::Bytes::from(request.signature.to_vec()),
                            factory: request
                                .prepared_calls
                                .data
                                .factory
                                .map(|factory| ethers::types::H160::from_slice(factory.as_bytes())),
                            factory_data: request.prepared_calls.data.factory_data.clone().map(
                                |factory_data| ethers::types::Bytes::from(factory_data.to_vec()),
                            ),
                            paymaster: request.prepared_calls.data.paymaster.map(|paymaster| {
                                ethers::types::H160::from_slice(paymaster.as_bytes())
                            }),
                            paymaster_verification_gas_limit: request
                                .prepared_calls
                                .data
                                .paymaster_verification_gas_limit
                                .map(|paymaster_verification_gas_limit| {
                                    ethers::types::U128::from(
                                        &paymaster_verification_gas_limit.to_be_bytes::<32>()[16..],
                                    )
                                }),
                            paymaster_post_op_gas_limit: request
                                .prepared_calls
                                .data
                                .paymaster_post_op_gas_limit
                                .map(|paymaster_post_op_gas_limit| {
                                    ethers::types::U128::from(
                                        &paymaster_post_op_gas_limit.to_be_bytes::<32>()[16..],
                                    )
                                }),
                            paymaster_data: request.prepared_calls.data.paymaster_data.clone().map(
                                |paymaster_data| {
                                    ethers::types::Bytes::from(paymaster_data.to_vec())
                                },
                            ),
                        },
                    };

                let response = match cosign::handler(
                    state.clone(),
                    Path({
                        format!(
                            "{}:{}",
                            chain_id.caip2_identifier(),
                            request
                                .prepared_calls
                                .data
                                .sender
                                .to_address()
                                .to_checksum(None)
                        )
                    }),
                    Query(CoSignQueryParams {
                        project_id: project_id.clone(),
                        version: None,
                    }),
                    SimpleRequestJson(cosign_request),
                )
                .await
                {
                    Ok(response) => response,
                    Err(e) => {
                        let response = e.into_response();
                        let status = response.status();
                        let response = String::from_utf8(
                            http_body_util::BodyExt::collect(response.into_body())
                                .await
                                // Lazy error handling here for now. We will refactor soon to avoid all this
                                .unwrap_or_default()
                                .to_bytes()
                                .to_vec(),
                        )
                        // Lazy error handling here for now. We will refactor soon to avoid all this
                        .unwrap_or_default();
                        let e = if status.is_server_error() {
                            SendPreparedCallsError::InternalError(
                                SendPreparedCallsInternalError::Cosign(response),
                            )
                        } else {
                            SendPreparedCallsError::Cosign(response)
                        };
                        return Err(e);
                    }
                };
                if !response.status().is_success() {
                    return Err(SendPreparedCallsError::InternalError(
                        SendPreparedCallsInternalError::CosignUnsuccessful(
                            http_body_util::BodyExt::collect(response.into_body())
                                .await
                                .map(|c| c.to_bytes()),
                        ),
                    ));
                }

                let response_json = serde_json::from_slice::<serde_json::Value>(
                    &http_body_util::BodyExt::collect(response.into_body())
                        .await
                        .map_err(|e| {
                            SendPreparedCallsError::InternalError(
                                SendPreparedCallsInternalError::CosignReadResponse(e),
                            )
                        })?
                        .to_bytes(),
                )
                .map_err(|e| {
                    SendPreparedCallsError::InternalError(
                        SendPreparedCallsInternalError::CosignParseResponse(e),
                    )
                })?;

                let signature_hex = response_json
                    .get("signature")
                    .ok_or_else(|| {
                        SendPreparedCallsError::InternalError(
                            SendPreparedCallsInternalError::CosignResponseMissingSignature,
                        )
                    })?
                    .as_str()
                    .ok_or_else(|| {
                        SendPreparedCallsError::InternalError(
                            SendPreparedCallsInternalError::CosignResponseSignatureNotString,
                        )
                    })?
                    .trim_start_matches("0x");

                hex::decode(signature_hex).map_err(|e| {
                    SendPreparedCallsError::InternalError(
                        SendPreparedCallsInternalError::CosignResponseSignatureNotHex(e),
                    )
                })?
            };

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

        // TODO refactor to call internal proxy function directly
        let provider = ProviderBuilder::default().on_http(
            format!(
                "https://rpc.walletconnect.org/v1?chainId={}&projectId={}&source={}",
                chain_id.caip2_identifier(),
                project_id,
                MessageSource::WalletSendPreparedCalls,
            )
            .parse()
            .unwrap(),
        );

        let irn_client = state
            .irn
            .as_ref()
            .ok_or(SendPreparedCallsError::InternalError(
                SendPreparedCallsInternalError::IrnNotConfigured,
            ))?;
        let context = get_session_context(
            format!(
                "{}:{}",
                chain_id.caip2_identifier(),
                request.prepared_calls.data.sender
            ),
            request.context,
            irn_client,
            &state.metrics,
        )
        .await
        .map_err(|e| match e {
            GetSessionContextError::PermissionNotFound(_, _) => {
                SendPreparedCallsError::PermissionNotFound
            }
            GetSessionContextError::InternalGetSessionContextError(e) => {
                SendPreparedCallsError::InternalError(
                    SendPreparedCallsInternalError::GetSessionContextError(e),
                )
            }
        })?
        .ok_or(SendPreparedCallsError::PciNotFound)?;

        let (_validator_address, signature) =
            split_permissions_context_and_check_validator(&context)
                .map_err(SendPreparedCallsError::SplitPermissionsContextAndCheckValidator)?;

        let DecodedSmartSessionSignature {
            mode,
            permission_id,
            signature: _,
            enable_session_data,
        } = decode_smart_session_signature(signature, account_type)
            .map_err(SendPreparedCallsError::DecodeSmartSessionSignature)?;

        let enable_session_data = match mode {
            SmartSessionMode::Enable | SmartSessionMode::UnsafeEnable => {
                // TODO refactor enum to avoid unwrap
                enable_session_data.unwrap()
            }
            SmartSessionMode::Use => {
                return Err(SendPreparedCallsError::PermissionContextUnsupportedModeUse)
            }
        };

        let signature = encode_use_or_enable_smart_session_signature(
            &provider,
            permission_id,
            request.prepared_calls.data.sender,
            account_type,
            cosign_signature,
            enable_session_data,
        )
        .await
        .map_err(SendPreparedCallsError::EncodeUseOrEnableSmartSessionSignature)?;

        let user_op = UserOperationV07 {
            signature,
            ..request.prepared_calls.data
        };

        // TODO refactor to use bundler_rpc_call directly: https://github.com/WalletConnect/blockchain-api/blob/8be3ca5b08dec2387ee2c2ffcb4b7ca739443bcb/src/handlers/bundler.rs#L62
        let bundler_url = format!(
            "https://rpc.walletconnect.org/v1/bundler?chainId={}&projectId={}&bundler=pimlico",
            chain_id.caip2_identifier(),
            project_id,
        );

        let bundler_client = BundlerClient::new(BundlerConfig::new(bundler_url.parse().unwrap()));

        let user_op_hash = bundler_client
            .send_user_operation(entry_point_config.address(), user_op)
            .await
            .map_err(SendPreparedCallsError::SendUserOperation)?;

        response.push(CallId(CallIdInner {
            chain_id: U64::from(chain_id.eip155_chain_id()),
            user_op_hash,
        }));
    }

    Ok(response)
}
