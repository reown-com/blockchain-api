use super::call_id::CallId;
use crate::{
    analytics::MessageSource,
    handlers::{RpcQueryParams, SdkInfoParams},
    state::AppState,
};
use alloy::{
    primitives::{Address, BlockHash, Bytes, TxHash, B256, U64, U8},
    providers::ProviderBuilder,
    rpc::{client::RpcClient, types::UserOperationReceipt},
};
use axum::extract::{ConnectInfo, Query, State};
use hyper::HeaderMap;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use thiserror::Error;
use tracing::error;
use wc::metrics::{future_metrics, FutureExt};
use yttrium::{chain::ChainId, erc4337::get_user_operation_receipt};

pub type GetCallsStatusParams = (CallId,);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCallsStatusResult {
    status: CallStatus,
    // TODO use tagged enum to avoid this Option
    receipts: Option<Vec<CallReceipt>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum CallStatus {
    Pending,
    Confirmed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallReceipt {
    logs: Vec<CallReceiptLog>,
    status: U8,
    chain_id: U64,
    block_hash: BlockHash,
    block_number: U64,
    gas_used: U64,
    transaction_hash: TxHash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallReceiptLog {
    address: Address,
    data: Bytes,
    topics: Vec<B256>,
}

#[derive(Error, Debug)]
pub enum GetCallsStatusError {
    #[error("Internal error")]
    InternalError(GetCallsStatusInternalError),
}

#[derive(Error, Debug)]
pub enum GetCallsStatusInternalError {
    #[error("UserOp operation get receipt error: {0}")]
    UserOperationReceiptError(String),
}

impl GetCallsStatusError {
    pub fn is_internal(&self) -> bool {
        matches!(self, GetCallsStatusError::InternalError(_))
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(flatten)]
    pub sdk_info: SdkInfoParams,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    project_id: String,
    request: GetCallsStatusParams,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query: Query<QueryParams>,
) -> Result<GetCallsStatusResult, GetCallsStatusError> {
    handler_internal(state, project_id, request, connect_info, headers, query)
        .with_metrics(future_metrics!("handler_task", "name" => "wallet_get_calls_status"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    project_id: String,
    request: GetCallsStatusParams,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query: Query<QueryParams>,
) -> Result<GetCallsStatusResult, GetCallsStatusError> {
    let chain_id = ChainId::new_eip155(request.0 .0.chain_id.to());
    let provider = ProviderBuilder::default().on_client(RpcClient::new(
        self_transport::SelfBundlerTransport {
            state: state.0.clone(),
            connect_info,
            headers,
            query: RpcQueryParams {
                chain_id: chain_id.into(),
                project_id,
                provider_id: None,
                session_id: None,
                source: Some(MessageSource::WalletGetCallsStatus),
                sdk_info: query.sdk_info.clone(),
            },
            chain_id,
        },
        false,
    ));

    let receipt = get_user_operation_receipt(&provider, request.0 .0.user_op_hash)
        .await
        .map_err(|e| {
            GetCallsStatusError::InternalError(
                GetCallsStatusInternalError::UserOperationReceiptError(e.to_string()),
            )
        })?;

    let receipt = match receipt {
        Some(receipt) => receipt,
        None => {
            return Ok(GetCallsStatusResult {
                status: CallStatus::Pending,
                receipts: None,
            })
        }
    };

    Ok(GetCallsStatusResult {
        status: if receipt.receipt.status() {
            CallStatus::Confirmed
        } else {
            CallStatus::Pending // FIXME this should be Error instead??
        },
        receipts: Some(vec![user_operation_receipt_to_call_receipt(
            request.0 .0.chain_id,
            receipt,
        )]),
    })
}

fn user_operation_receipt_to_call_receipt(
    chain_id: U64,
    receipt: UserOperationReceipt,
) -> CallReceipt {
    CallReceipt {
        logs: receipt
            .logs
            .into_iter()
            .map(|log| CallReceiptLog {
                address: log.address(),
                topics: log.topics().to_vec(),
                data: log.data().data.clone(),
            })
            .collect(),
        status: match receipt.receipt.status() {
            true => U8::from(1),
            false => U8::from(0),
        },
        chain_id,
        block_hash: receipt.receipt.block_hash.unwrap_or_default(), // FIXME
        block_number: receipt
            .receipt
            .block_number
            .map(U64::from)
            .unwrap_or_default(), // FIXME
        gas_used: U64::from(receipt.receipt.gas_used),
        transaction_hash: receipt.receipt.transaction_hash,
    }
}

mod self_transport {
    use {
        crate::{
            error::RpcError, handlers::RpcQueryParams, json_rpc::JSON_RPC_VERSION,
            providers::SupportedBundlerOps, state::AppState, utils::crypto::disassemble_caip2,
        },
        alloy::{
            rpc::json_rpc::{RequestPacket, Response, ResponsePacket},
            transports::{TransportError, TransportErrorKind, TransportFut},
        },
        hyper::HeaderMap,
        std::{net::SocketAddr, sync::Arc, task::Poll},
        tower::Service,
        yttrium::chain::ChainId,
    };

    #[derive(thiserror::Error, Debug)]
    pub enum SelfBundlerTransportError {
        #[error("RPC error: {0}")]
        Rpc(RpcError),

        #[error("Parse params: {0}")]
        ParseParams(serde_json::Error),

        #[error("No result")]
        NoResult,

        #[error("Parse result: {0}")]
        ParseResult(serde_json::Error),
    }

    #[derive(Clone)]
    pub struct SelfBundlerTransport {
        pub state: Arc<AppState>,
        pub connect_info: SocketAddr,
        pub query: RpcQueryParams,
        pub headers: HeaderMap,
        pub chain_id: ChainId,
    }

    impl Service<RequestPacket> for SelfBundlerTransport {
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
            let _connect_info = self.connect_info;
            let _query = self.query.clone();
            let _headers = self.headers.clone();
            let caip2_identifier = self.chain_id.caip2_identifier();

            Box::pin(async move {
                // TODO handle batch
                let req = match req {
                    RequestPacket::Single(req) => req,
                    RequestPacket::Batch(_) => unimplemented!(),
                };

                let method =
                    serde_json::from_value::<SupportedBundlerOps>(serde_json::json!(req.method()))
                        .map_err(|_| TransportErrorKind::custom_str("Unsupported method"))?;
                let params = serde_json::from_str(
                    req.params()
                        .ok_or_else(|| TransportErrorKind::custom_str("Params is null"))?
                        .get(),
                )
                .map_err(|e| {
                    TransportErrorKind::custom(SelfBundlerTransportError::ParseParams(e))
                })?;

                let (_, eip155_chain_id) = disassemble_caip2(&caip2_identifier)
                    .map_err(|_| TransportErrorKind::custom_str("Failed to parse CAIP2 chainId"))?;
                let response = state
                    .providers
                    .bundler_ops_provider
                    .bundler_rpc_call(
                        &eip155_chain_id,
                        req.id().clone(),
                        JSON_RPC_VERSION.clone(),
                        &method,
                        params,
                    )
                    .await
                    .map_err(|e| TransportErrorKind::custom(SelfBundlerTransportError::Rpc(e)))?;
                // TODO check for error
                let body = serde_json::to_string(response.get("result").ok_or_else(|| {
                    TransportErrorKind::custom(SelfBundlerTransportError::NoResult)
                })?)
                .map_err(|e| {
                    TransportErrorKind::custom(SelfBundlerTransportError::ParseResult(e))
                })?;

                Ok(ResponsePacket::Single(Response {
                    id: req.id().clone(),
                    payload: alloy::rpc::json_rpc::ResponsePayload::Success(
                        serde_json::value::RawValue::from_string(body).unwrap(),
                    ),
                }))
            })
        }
    }
}

// TODO test case:
// - check receipt contents, e.g. confirmed, pending. Unsuccessful txn
// - what about missing Option in alloy's getUserOperationReceipt

#[cfg(test)]
#[cfg(feature = "test-mock-bundler")]
mod tests {
    use crate::{
        handlers::json_rpc::{
            call_id::{CallId, CallIdInner},
            get_calls_status::{CallStatus, GetCallsStatusResult},
            handler::WALLET_GET_CALLS_STATUS,
        },
        providers::mock_alto::MockAltoUrls,
        test_helpers::spawn_blockchain_api_with_params,
    };
    use alloy::{
        primitives::{Bytes, Uint, U256, U64, U8},
        providers::{Provider, ProviderBuilder},
        signers::local::LocalSigner,
    };
    use yttrium::{
        call::{send::safe_test::send_transactions, Call},
        config::Config,
        smart_accounts::safe::{get_account_address, Owners},
        test_helpers::{anvil_faucet, use_faucet},
    };

    #[tokio::test]
    async fn test_get_calls_status() {
        // let anvil = Anvil::new().spawn();
        let config = Config::local();
        let provider =
            ProviderBuilder::default().on_http(config.endpoints.rpc.base_url.parse().unwrap());
        let faucet = anvil_faucet(&provider).await;

        let destination = LocalSigner::random();
        let balance = provider.get_balance(destination.address()).await.unwrap();
        assert_eq!(balance, Uint::from(0));

        let owner = LocalSigner::random();
        let sender_address = get_account_address(
            provider.clone(),
            Owners {
                owners: vec![owner.address()],
                threshold: 1,
            },
        )
        .await;

        use_faucet(&provider, faucet, U256::from(1), sender_address.into()).await;

        let transaction = vec![Call {
            to: destination.address(),
            value: Uint::from(1),
            input: Bytes::new(),
        }];

        let receipt = send_transactions(transaction, owner.clone(), None, None, config.clone())
            .await
            .unwrap();
        assert!(receipt.success);

        let balance = provider.get_balance(destination.address()).await.unwrap();
        assert_eq!(balance, Uint::from(1));

        let url = spawn_blockchain_api_with_params(crate::test_helpers::Params {
            validate_project_id: false,
            override_bundler_urls: Some(MockAltoUrls {
                bundler_url: config.endpoints.bundler.base_url.parse().unwrap(),
                paymaster_url: config.endpoints.paymaster.base_url.parse().unwrap(),
            }),
        })
        .await;
        let mut endpoint = url.join("/v1/wallet").unwrap();
        endpoint.query_pairs_mut().append_pair("projectId", "test");
        let provider = ProviderBuilder::new().on_http(endpoint);
        let result = provider
            .client()
            .request::<_, GetCallsStatusResult>(
                WALLET_GET_CALLS_STATUS,
                (CallId(CallIdInner {
                    chain_id: U64::from(11155111),
                    user_op_hash: receipt.user_op_hash,
                }),),
            )
            .await
            .unwrap();
        assert_eq!(result.status, CallStatus::Confirmed);
        let receipts = result.receipts.unwrap();
        assert_eq!(receipts.len(), 1);
        let receipt = receipts.first().unwrap();
        assert_eq!(receipt.status, U8::from(1));
        assert_eq!(receipt.chain_id, U64::from(11155111));
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_calls_status_failed() {
        // let anvil = Anvil::new().spawn();
        let config = Config::local();
        let provider =
            ProviderBuilder::new().on_http(config.endpoints.rpc.base_url.parse().unwrap());
        let _faucet = anvil_faucet(&provider);

        let destination = LocalSigner::random();
        let balance = provider.get_balance(destination.address()).await.unwrap();
        assert_eq!(balance, Uint::from(0));

        let owner = LocalSigner::random();
        let _sender_address = get_account_address(
            provider.clone(),
            Owners {
                owners: vec![owner.address()],
                threshold: 1,
            },
        )
        .await;

        let transaction = vec![Call {
            to: destination.address(),
            value: Uint::from(1),
            input: Bytes::new(),
        }];

        let receipt = send_transactions(transaction, owner.clone(), None, None, config.clone())
            .await
            .unwrap();
        assert!(!receipt.success);

        let balance = provider.get_balance(destination.address()).await.unwrap();
        assert_eq!(balance, Uint::from(1));

        let url = spawn_blockchain_api_with_params(crate::test_helpers::Params {
            validate_project_id: false,
            override_bundler_urls: Some(MockAltoUrls {
                bundler_url: config.endpoints.bundler.base_url.parse().unwrap(),
                paymaster_url: config.endpoints.paymaster.base_url.parse().unwrap(),
            }),
        })
        .await;
        let mut endpoint = url.join("/v1/wallet").unwrap();
        endpoint.query_pairs_mut().append_pair("projectId", "test");
        let provider = ProviderBuilder::new().on_http(endpoint);
        let result = provider
            .client()
            .request::<_, GetCallsStatusResult>(
                WALLET_GET_CALLS_STATUS,
                (CallId(CallIdInner {
                    chain_id: U64::from(11155111),
                    user_op_hash: receipt.user_op_hash,
                }),),
            )
            .await
            .unwrap();
        assert_eq!(result.status, CallStatus::Confirmed);
        let receipts = result.receipts.unwrap();
        assert_eq!(receipts.len(), 1);
        let receipt = receipts.first().unwrap();
        assert_eq!(receipt.status, U8::from(1));
        assert_eq!(receipt.chain_id, U64::from(11155111));
    }
}
