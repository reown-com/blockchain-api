use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain,
    crate::{context::ServerContext, utils::send_jsonrpc_request, JSONRPC_VERSION},
    hyper::{Client, StatusCode},
    hyper_tls::HttpsConnector,
    rpc_proxy::providers::ProviderKind,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn pokt_provider_eip155(ctx: &mut ServerContext) {
    // Avax mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Pokt,
        "eip155:43114",
        "0xa86a",
    )
    .await;

    // Gnosis
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Pokt,
        "eip155:100",
        "0x64",
    )
    .await;

    // Base mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Pokt,
        "eip155:8453",
        "0x2105",
    )
    .await;

    // Base testnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Pokt,
        "eip155:84531",
        "0x14a33",
    )
    .await;

    // Binance mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Pokt,
        "eip155:56",
        "0x38",
    )
    .await;

    // Ethereum
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Pokt,
        "eip155:1",
        "0x1",
    )
    .await;

    // Goerli
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Pokt,
        "eip155:5",
        "0x5",
    )
    .await;

    // Optimism
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Pokt,
        "eip155:10",
        "0xa",
    )
    .await;

    // Arbitrum
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Pokt,
        "eip155:42161",
        "0xa4b1",
    )
    .await;

    // Polygon mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Pokt,
        "eip155:137",
        "0x89",
    )
    .await;

    // Polygon mumbai
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Pokt,
        "eip155:80001",
        "0x13881",
    )
    .await;

    // Polygon zkevm
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Pokt,
        "eip155:1101",
        "0x44d",
    )
    .await;

    // Polygon celo
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Pokt,
        "eip155:42220",
        "0xa4ec",
    )
    .await;

    // Klaytn mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Pokt,
        "eip155:8217",
        "0x2019",
    )
    .await;
}

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn pokt_provider_solana_mainnet(ctx: &mut ServerContext) {
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
