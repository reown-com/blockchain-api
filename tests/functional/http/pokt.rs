use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain,
    crate::{context::ServerContext, utils::send_jsonrpc_request, JSONRPC_VERSION},
    hyper::{Client, StatusCode},
    hyper_tls::HttpsConnector,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
async fn pokt_provider(ctx: &mut ServerContext) {
    // Avax
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:43114", "0xa86a").await;

    // Binance mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:56", "0x38").await;

    // Gnosis
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:100", "0x64").await
}

#[test_context(ServerContext)]
#[tokio::test]
async fn solana_mainnet(ctx: &mut ServerContext) {
    let addr = format!(
        "{}/v1?projectId={}&chainId=",
        ctx.server.public_addr, ctx.server.project_id
    );

    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
    let request = jsonrpc::Request {
        method: "getHealth",
        params: &[],
        id: serde_json::Value::Number(1.into()),
        jsonrpc: JSONRPC_VERSION,
    };

    let (status, rpc_response) = send_jsonrpc_request(
        client,
        addr,
        "solana:4sgjmw1sunhzsxgspuhpqldx6wiyjntz",
        request,
    )
    .await;

    // Verify that HTTP communication was successful
    assert_eq!(status, StatusCode::OK);

    // Verify there was no error in rpc
    assert!(rpc_response.error.is_none());

    // Check chainId
    assert_eq!(rpc_response.result::<String>().unwrap(), "ok")
}
