use super::call_id::CallId;
use crate::{
    analytics::MessageSource,
    handlers::{RpcQueryParams, HANDLER_TASK_METRICS},
    state::AppState,
};
use alloy::{
    network::Ethereum,
    primitives::{Address, BlockHash, Bytes, TxHash, B256, U128, U64, U8},
    providers::RootProvider,
    rpc::client::RpcClient,
};
use axum::{
    extract::{ConnectInfo, State},
    response::{IntoResponse, Response},
    Json,
};
use hyper::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use thiserror::Error;
use tracing::error;
use wc::future::FutureExt;
use yttrium::{
    bundler::{client::CustomErc4337Api, models::user_operation_receipt::UserOperationReceipt},
    chain::ChainId,
};

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
    gas_used: U128,
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
pub enum GetCallsStatusInternalError {}

impl IntoResponse for GetCallsStatusError {
    fn into_response(self) -> Response {
        #[allow(unreachable_patterns)] // TODO remove
        match self {
            Self::InternalError(e) => {
                error!("HTTP server error: (get_calls_status) {e:?}");
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
    project_id: String,
    request: GetCallsStatusParams,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<GetCallsStatusResult, GetCallsStatusError> {
    handler_internal(state, project_id, request, connect_info, headers)
        .with_metrics(HANDLER_TASK_METRICS.with_name("wallet_prepare_calls"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    project_id: String,
    request: GetCallsStatusParams,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<GetCallsStatusResult, GetCallsStatusError> {
    let chain_id = ChainId::new_eip155(request.0 .0.chain_id.to());
    let provider = RootProvider::<_, Ethereum>::new(RpcClient::new(
        self_transport::SelfBundlerTransport {
            state: state.0.clone(),
            connect_info,
            headers,
            query: RpcQueryParams {
                chain_id: chain_id.caip2_identifier(),
                project_id,
                provider_id: None,
                source: Some(MessageSource::WalletGetCallsStatus),
            },
            chain_id,
        },
        false,
    ));

    let receipt = provider
        .get_user_operation_receipt(request.0 .0.user_op_hash)
        .await
        .unwrap();
    // TODO handle None as CallStatus::Pending

    Ok(GetCallsStatusResult {
        status: if receipt.receipt.status == U8::from(1) {
            CallStatus::Confirmed
        } else {
            CallStatus::Pending
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
                address: log.address,
                topics: log.topics,
                data: log.data,
            })
            .collect(),
        status: receipt.receipt.status,
        chain_id,
        block_hash: receipt.receipt.block_hash,
        block_number: receipt.receipt.block_number,
        gas_used: receipt.receipt.gas_used,
        transaction_hash: receipt.receipt.transaction_hash,
    }
}

mod self_transport {
    use {
        crate::{
            error::RpcError, handlers::RpcQueryParams, json_rpc::JSON_RPC_VERSION,
            providers::SupportedBundlerOps, state::AppState,
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

                let response = state
                    .providers
                    .bundler_ops_provider
                    .bundler_rpc_call(
                        &caip2_identifier,
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
        handlers::wallet::{
            call_id::{CallId, CallIdInner},
            get_calls_status::{CallStatus, GetCallsStatusResult},
            handler::WALLET_GET_CALLS_STATUS,
        },
        providers::mock_alto::MockAltoUrls,
        test_helpers::spawn_blockchain_api_with_params,
    };
    use alloy::{
        network::Ethereum,
        primitives::{Bytes, Uint, U256, U64, U8},
        providers::{ext::AnvilApi, Provider, ReqwestProvider},
        signers::{k256::ecdsa::SigningKey, local::LocalSigner},
    };
    use reqwest::IntoUrl;
    use yttrium::{
        config::Config,
        smart_accounts::safe::{get_account_address, Owners},
        test_helpers::use_faucet,
        transaction::{send::safe_test::send_transactions, Transaction},
    };

    pub async fn anvil_faucet<T: IntoUrl>(url: T) -> LocalSigner<SigningKey> {
        println!(
            "RPC_PROXY_POSTGRES_URI b.1: {}",
            std::env::var("RPC_PROXY_POSTGRES_URI").unwrap()
        );
        let faucet = LocalSigner::random();
        std::env::vars().for_each(|(k, v)| println!("{}: {}", k, v));
        println!(
            "RPC_PROXY_POSTGRES_URI b.2: {:?}",
            std::env::var_os("RPC_PROXY_POSTGRES_URI").unwrap()
        );
        println!(
            "RPC_PROXY_POSTGRES_URI b.2: {}",
            std::env::var("RPC_PROXY_POSTGRES_URI").unwrap()
        );
        let provider = ReqwestProvider::<Ethereum>::new_http(url.into_url().unwrap());
        println!(
            "RPC_PROXY_POSTGRES_URI b.3: {}",
            std::env::var("RPC_PROXY_POSTGRES_URI").unwrap()
        );
        provider
            .anvil_set_balance(faucet.address(), U256::MAX)
            .await
            .unwrap();
        println!(
            "RPC_PROXY_POSTGRES_URI b.4: {}",
            std::env::var("RPC_PROXY_POSTGRES_URI").unwrap()
        );
        faucet
    }

    #[tokio::test]
    async fn test_get_calls_status() {
        println!(
            "RPC_PROXY_POSTGRES_URI -1: {}",
            std::env::var("RPC_PROXY_POSTGRES_URI").unwrap()
        );
        // let anvil = Anvil::new().spawn();
        let config = Config::local();
        println!(
            "RPC_PROXY_POSTGRES_URI a.2: {}",
            std::env::var("RPC_PROXY_POSTGRES_URI").unwrap()
        );
        let faucet = anvil_faucet(config.endpoints.rpc.base_url.clone()).await;
        println!(
            "RPC_PROXY_POSTGRES_URI a.3: {}",
            std::env::var("RPC_PROXY_POSTGRES_URI").unwrap()
        );

        let provider =
            ReqwestProvider::<Ethereum>::new_http(config.endpoints.rpc.base_url.parse().unwrap());
        println!(
            "RPC_PROXY_POSTGRES_URI a.4: {}",
            std::env::var("RPC_PROXY_POSTGRES_URI").unwrap()
        );

        let destination = LocalSigner::random();
        let balance = provider.get_balance(destination.address()).await.unwrap();
        assert_eq!(balance, Uint::from(0));
        println!(
            "RPC_PROXY_POSTGRES_URI a.5: {}",
            std::env::var("RPC_PROXY_POSTGRES_URI").unwrap()
        );

        let owner = LocalSigner::random();
        let sender_address = get_account_address(
            provider.clone(),
            Owners {
                owners: vec![owner.address()],
                threshold: 1,
            },
        )
        .await;
        println!(
            "RPC_PROXY_POSTGRES_URI a.6: {}",
            std::env::var("RPC_PROXY_POSTGRES_URI").unwrap()
        );

        use_faucet(
            provider.clone(),
            faucet.clone(),
            U256::from(1),
            sender_address.into(),
        )
        .await;
        println!(
            "RPC_PROXY_POSTGRES_URI a.7: {}",
            std::env::var("RPC_PROXY_POSTGRES_URI").unwrap()
        );

        let transaction = vec![Transaction {
            to: destination.address(),
            value: Uint::from(1),
            data: Bytes::new(),
        }];

        let receipt = send_transactions(transaction, owner.clone(), None, None, config.clone())
            .await
            .unwrap();
        assert!(receipt.success);
        println!(
            "RPC_PROXY_POSTGRES_URI a.8: {}",
            std::env::var("RPC_PROXY_POSTGRES_URI").unwrap()
        );

        let balance = provider.get_balance(destination.address()).await.unwrap();
        assert_eq!(balance, Uint::from(1));

        println!(
            "RPC_PROXY_POSTGRES_URI a.199: {}",
            std::env::var("RPC_PROXY_POSTGRES_URI").unwrap()
        );
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
        let provider = ReqwestProvider::<Ethereum>::new_http(endpoint);
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
        let _faucet = anvil_faucet(config.endpoints.rpc.base_url.clone()).await;

        let provider =
            ReqwestProvider::<Ethereum>::new_http(config.endpoints.rpc.base_url.parse().unwrap());

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

        let transaction = vec![Transaction {
            to: destination.address(),
            value: Uint::from(1),
            data: Bytes::new(),
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
        let provider = ReqwestProvider::<Ethereum>::new_http(endpoint);
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
