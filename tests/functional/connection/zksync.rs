use {
    crate::{context::ServerContext, utils::send_jsonrpc_request, JSONRPC_VERSION},
    hyper::{Client, StatusCode},
    hyper_tls::HttpsConnector,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_324_zksync_mainnet(ctx: &mut ServerContext) {
    let addr = format!(
        "http://{}/v1?projectId={}&chainId=",
        ctx.server.public_addr, ctx.server.project_id
    );

    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
    let request = jsonrpc::Request {
        method: "eth_chainId",
        params: &[],
        id: serde_json::Value::Number(1.into()),
        jsonrpc: JSONRPC_VERSION,
    };

    let (status, rpc_response) = send_jsonrpc_request(client, addr, "eip155:324", request).await;

    // Verify that HTTP communication was successful
    assert_eq!(status, StatusCode::OK);

    // Verify there was no error in rpc
    assert!(rpc_response.error.is_none());

    // Check chainId
    assert_eq!(rpc_response.result::<String>().unwrap(), "0x144")
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_280_zksync_testnet(ctx: &mut ServerContext) {
    let addr = format!(
        "http://{}/v1?projectId={}&chainId=",
        ctx.server.public_addr, ctx.server.project_id
    );

    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
    let request = jsonrpc::Request {
        method: "eth_chainId",
        params: &[],
        id: serde_json::Value::Number(1.into()),
        jsonrpc: JSONRPC_VERSION,
    };

    let (status, rpc_response) = send_jsonrpc_request(client, addr, "eip155:280", request).await;

    // Verify that HTTP communication was successful
    assert_eq!(status, StatusCode::OK);

    // Verify there was no error in rpc
    assert!(rpc_response.error.is_none());

    // Check chainId
    assert_eq!(rpc_response.result::<String>().unwrap(), "0x118")
}
