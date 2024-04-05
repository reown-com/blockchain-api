use {
    crate::{context::ServerContext, utils::send_jsonrpc_request, JSONRPC_VERSION},
    hyper::StatusCode,
    rpc_proxy::{handlers::history::HistoryResponseBody, providers::ProviderKind},
    test_context::test_context,
};

pub(crate) mod aurora;
pub(crate) mod base;
pub(crate) mod binance;
pub(crate) mod getblock;
pub(crate) mod infura;
pub(crate) mod mantle;
pub(crate) mod near;
pub(crate) mod pokt;
pub(crate) mod publicnode;
pub(crate) mod quicknode;
pub(crate) mod zksync;
pub(crate) mod zora;

async fn check_if_rpc_is_responding_correctly_for_supported_chain(
    ctx: &ServerContext,
    provider_id: &ProviderKind,
    chaind_id: &str,
    expected_id: &str,
) {
    let addr = format!(
        "{}/v1/?projectId={}&providerId={}&chainId=",
        ctx.server.public_addr, ctx.server.project_id, provider_id
    );

    let request = jsonrpc::Request {
        method: "eth_chainId",
        params: &[],
        id: serde_json::Value::Number(1.into()),
        jsonrpc: JSONRPC_VERSION,
    };

    let (status, rpc_response) = send_jsonrpc_request(addr, chaind_id, &request).await;

    match status {
        StatusCode::OK => {
            // Verify there was no error in rpc
            assert!(rpc_response.error.is_none());

            // Check chainId
            assert_eq!(rpc_response.result::<String>().unwrap(), expected_id)
        }
        _ => panic!("Unexpected status code: {}", status),
    };
}

async fn check_if_rpc_is_responding_correctly_for_near_protocol(
    ctx: &ServerContext,
    provider_id: &ProviderKind,
) {
    let addr = format!(
        "{}/v1/?projectId={}&providerId={}&chainId=",
        ctx.server.public_addr, ctx.server.project_id, provider_id
    );

    let request = jsonrpc::Request {
        method: "EXPERIMENTAL_genesis_config",
        params: &[],
        id: serde_json::Value::Number(1.into()),
        jsonrpc: JSONRPC_VERSION,
    };

    let (status, rpc_response) = send_jsonrpc_request(addr, "near:mainnet", &request).await;

    #[derive(serde::Deserialize)]
    struct GenesisConfig {
        chain_id: String,
    }

    match status {
        StatusCode::OK => {
            // Verify there was no error in rpc
            assert!(rpc_response.error.is_none());

            // Check chainId
            assert_eq!(
                rpc_response.result::<GenesisConfig>().unwrap().chain_id,
                "mainnet"
            )
        }
        _ => panic!("Unexpected status code: {}", status),
    };
}

async fn check_if_rpc_is_responding_correctly_for_solana(
    ctx: &ServerContext,
    chain_id: &str,
    provider_id: &ProviderKind,
) {
    let addr = format!(
        "{}/v1/?projectId={}&providerId={}&chainId=",
        ctx.server.public_addr, ctx.server.project_id, provider_id
    );

    let request = jsonrpc::Request {
        method: "getHealth",
        params: &[],
        id: serde_json::Value::Number(1.into()),
        jsonrpc: JSONRPC_VERSION,
    };

    let (status, rpc_response) =
        send_jsonrpc_request(addr, &format!("solana:{}", chain_id), &request).await;

    // Verify that HTTP communication was successful
    assert_eq!(status, StatusCode::OK);

    // Verify there was no error in rpc
    assert!(rpc_response.error.is_none());

    // Check chainId
    assert_eq!(rpc_response.result::<String>().unwrap(), "ok")
}

#[test_context(ServerContext)]
#[tokio::test]
async fn health_check(ctx: &mut ServerContext) {
    let addr = format!("{}/health", ctx.server.public_addr);

    let response = reqwest::get(addr).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK)
}

#[test_context(ServerContext)]
#[tokio::test]
async fn account_history_check(ctx: &mut ServerContext) {
    let account = "0xf3ea39310011333095CFCcCc7c4Ad74034CABA63";
    let project_id = ctx.server.project_id.clone();
    let addr = format!(
        "{}/v1/account/{}/history?projectId={}",
        ctx.server.public_addr, account, project_id
    );

    let response = reqwest::get(addr).await.unwrap();
    let status = response.status();
    assert_eq!(status, StatusCode::OK);

    let json_response = response
        .json::<HistoryResponseBody>()
        .await
        .expect("Failed to parse response body");
    assert!(!json_response.data.is_empty());
}
