use {
    super::{
        call_id::{CallId, CallIdInner},
        prepare_calls::{
            decode_smart_session_signature,
            encode_use_or_enable_smart_session_signature,
            split_permissions_context_and_check_validator,
            AccountType,
            DecodedSmartSessionSignature,
            PrepareCallsError,
        },
        types::PreparedCalls,
    },
    crate::{
        analytics::MessageSource,
        handlers::{
            sessions::{
                cosign::{self, CoSignQueryParams},
                get::{
                    get_session_context,
                    GetSessionContextError,
                    InternalGetSessionContextError,
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
    hyper::body::to_bytes,
    parquet::data_type::AsBytes,
    serde::{Deserialize, Serialize},
    std::sync::Arc,
    thiserror::Error,
    tracing::error,
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
    CosignUnsuccessful(std::result::Result<hyper::body::Bytes, axum::Error>),

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
    tracing::debug!("=== SEND PREPARED CALLS HANDLER STARTED ===");
    tracing::debug!("Processing {} prepared call(s)", request.len());

    let mut response = Vec::with_capacity(request.len());
    for (i, request) in request.into_iter().enumerate() {
        tracing::debug!("=== PROCESSING PREPARED CALL {} ===", i + 1);
        let chain_id = ChainId::new_eip155(request.prepared_calls.chain_id.to::<u64>());
        tracing::debug!("Chain ID: {}", chain_id.caip2_identifier());
        tracing::debug!("Sender: {}", request.prepared_calls.data.sender);
        tracing::debug!("Context UUID: {}", request.context);

        tracing::debug!("=== STARTING COSIGN PROCESS ===");
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

                tracing::debug!("Cosign request PCI: {}", cosign_request.pci);
                tracing::debug!(
                    "Cosign request user_op sender: {:?}",
                    cosign_request.user_op.sender
                );
                tracing::debug!(
                    "Cosign request user_op nonce: {:?}",
                    cosign_request.user_op.nonce
                );
                tracing::debug!(
                    "Cosign request user_op signature length: {} bytes",
                    cosign_request.user_op.signature.len()
                );

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
                    Ok(response) => {
                        tracing::debug!("Cosign request successful");
                        response
                    }
                    Err(e) => {
                        tracing::error!("Cosign request failed: {:?}", e);
                        let response = e.into_response();
                        let status = response.status();
                        let response = String::from_utf8(
                            to_bytes(response.into_body())
                                .await
                                // Lazy error handling here for now. We will refactor soon to avoid all this
                                .unwrap_or_default()
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
                    tracing::error!("Cosign response not successful: {}", response.status());
                    return Err(SendPreparedCallsError::InternalError(
                        SendPreparedCallsInternalError::CosignUnsuccessful(
                            to_bytes(response.into_body()).await,
                        ),
                    ));
                }

                let response_json = serde_json::from_slice::<serde_json::Value>(
                    &to_bytes(response.into_body()).await.map_err(|e| {
                        tracing::error!("Failed to read cosign response: {:?}", e);
                        SendPreparedCallsError::InternalError(
                            SendPreparedCallsInternalError::CosignReadResponse(e),
                        )
                    })?,
                )
                .map_err(|e| {
                    tracing::error!("Failed to parse cosign response JSON: {:?}", e);
                    SendPreparedCallsError::InternalError(
                        SendPreparedCallsInternalError::CosignParseResponse(e),
                    )
                })?;

                let signature_hex = response_json
                    .get("signature")
                    .ok_or_else(|| {
                        tracing::error!("Cosign response missing 'signature' field");
                        SendPreparedCallsError::InternalError(
                            SendPreparedCallsInternalError::CosignResponseMissingSignature,
                        )
                    })?
                    .as_str()
                    .ok_or_else(|| {
                        tracing::error!("Cosign response 'signature' field is not a string");
                        SendPreparedCallsError::InternalError(
                            SendPreparedCallsInternalError::CosignResponseSignatureNotString,
                        )
                    })?
                    .trim_start_matches("0x");

                tracing::debug!("Cosign signature hex length: {} chars", signature_hex.len());
                tracing::debug!("Cosign signature: 0x{}", signature_hex);

                let signature_bytes = hex::decode(signature_hex).map_err(|e| {
                    tracing::error!("Failed to decode cosign signature hex: {:?}", e);
                    SendPreparedCallsError::InternalError(
                        SendPreparedCallsInternalError::CosignResponseSignatureNotHex(e),
                    )
                })?;

                tracing::debug!("=== COSIGN PROCESS COMPLETE ===");
                tracing::debug!("Cosign signature bytes length: {}", signature_bytes.len());
                tracing::debug!("Cosign signature hex: 0x{}", hex::encode(&signature_bytes));

                // Debug: Check if this looks like ABI-encoded or concatenated format
                if signature_bytes.len() >= 32 {
                    tracing::debug!("First 32 bytes: 0x{}", hex::encode(&signature_bytes[0..32]));
                    if signature_bytes.len() >= 64 {
                        tracing::debug!(
                            "Second 32 bytes: 0x{}",
                            hex::encode(&signature_bytes[32..64])
                        );
                    }
                }

                signature_bytes
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
        tracing::debug!("Account type: {:?}", account_type);

        let entry_point_config = EntryPointConfig {
            chain_id,
            version: EntryPointVersion::V07,
        };
        tracing::debug!("Entry point: {}", entry_point_config.address());

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

        tracing::debug!("=== RETRIEVING SESSION CONTEXT ===");
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
        .map_err(|e| {
            tracing::error!("Failed to get session context: {:?}", e);
            match e {
                GetSessionContextError::PermissionNotFound(_, _) => {
                    SendPreparedCallsError::PermissionNotFound
                }
                GetSessionContextError::InternalGetSessionContextError(e) => {
                    SendPreparedCallsError::InternalError(
                        SendPreparedCallsInternalError::GetSessionContextError(e),
                    )
                }
            }
        })?
        .ok_or_else(|| {
            tracing::error!("Session context not found (PCI not found)");
            SendPreparedCallsError::PciNotFound
        })?;

        tracing::debug!("Session context retrieved, length: {} bytes", context.len());

        tracing::debug!("=== PROCESSING SESSION SIGNATURE ===");
        let (_validator_address, signature) =
            split_permissions_context_and_check_validator(&context).map_err(|e| {
                tracing::error!("Failed to split permissions context: {:?}", e);
                SendPreparedCallsError::SplitPermissionsContextAndCheckValidator(e)
            })?;

        let DecodedSmartSessionSignature {
            mode,
            permission_id,
            signature: _,
            enable_session_data,
        } = decode_smart_session_signature(signature, account_type).map_err(|e| {
            tracing::error!("Failed to decode smart session signature: {:?}", e);
            SendPreparedCallsError::DecodeSmartSessionSignature(e)
        })?;

        tracing::debug!("Decoded smart session signature mode: {:?}", mode);
        tracing::debug!("Permission ID: {:?}", permission_id);

        let enable_session_data = match mode {
            SmartSessionMode::Enable | SmartSessionMode::UnsafeEnable => {
                tracing::debug!("Using Enable/UnsafeEnable mode with session data");
                // TODO refactor enum to avoid unwrap
                enable_session_data.unwrap()
            }
            SmartSessionMode::Use => {
                tracing::error!("Use mode is not supported in send_prepared_calls");
                return Err(SendPreparedCallsError::PermissionContextUnsupportedModeUse);
            }
        };

        tracing::debug!("=== ENCODING FINAL SIGNATURE ===");
        let signature = encode_use_or_enable_smart_session_signature(
            &provider,
            permission_id,
            request.prepared_calls.data.sender,
            account_type,
            cosign_signature,
            enable_session_data,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to encode smart session signature: {:?}", e);
            SendPreparedCallsError::EncodeUseOrEnableSmartSessionSignature(e)
        })?;

        tracing::debug!("Final encoded signature length: {} bytes", signature.len());

        let user_op = UserOperationV07 {
            signature,
            ..request.prepared_calls.data
        };

        tracing::debug!("=== SENDING USER OPERATION ===");
        // TODO refactor to use bundler_rpc_call directly: https://github.com/WalletConnect/blockchain-api/blob/8be3ca5b08dec2387ee2c2ffcb4b7ca739443bcb/src/handlers/bundler.rs#L62
        let bundler_url = format!(
            "https://rpc.walletconnect.org/v1/bundler?chainId={}&projectId={}&bundler=pimlico",
            chain_id.caip2_identifier(),
            project_id,
        );
        tracing::debug!("Bundler URL: {}", bundler_url);

        let bundler_client = BundlerClient::new(BundlerConfig::new(bundler_url.parse().unwrap()));

        tracing::debug!("Sending user operation to bundler...");
        let user_op_hash = bundler_client
            .send_user_operation(entry_point_config.address(), user_op)
            .await
            .map_err(|e| {
                tracing::error!("Failed to send user operation: {:?}", e);
                SendPreparedCallsError::SendUserOperation(e)
            })?;

        tracing::debug!("=== USER OPERATION SENT SUCCESSFULLY ===");
        tracing::debug!("User operation hash: 0x{}", hex::encode(&user_op_hash));

        response.push(CallId(CallIdInner {
            chain_id: U64::from(chain_id.eip155_chain_id()),
            user_op_hash,
        }));

        tracing::debug!("=== PREPARED CALL {} COMPLETE ===", i + 1);
    }

    tracing::debug!("=== SEND PREPARED CALLS HANDLER COMPLETE ===");
    tracing::debug!("Successfully processed {} prepared call(s)", response.len());
    Ok(response)
}
