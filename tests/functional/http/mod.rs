use {
    crate::{context::ServerContext, utils::send_jsonrpc_request, JSONRPC_VERSION},
    hyper::{Body, Client, Method, Request, StatusCode},
    hyper_tls::HttpsConnector,
    test_context::test_context,
};

pub(crate) mod binance;
pub(crate) mod infura;
pub(crate) mod pokt;
pub(crate) mod zksync;

async fn check_if_rpc_is_responding_correctly_for_supported_chain(
    ctx: &mut ServerContext,
    chaind_id: &str,
    expected_id: &str,
) {
    let addr = format!(
        "{}/v1/?projectId={}&chainId=",
        ctx.server.public_addr, ctx.server.project_id
    );

    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
    let request = jsonrpc::Request {
        method: "eth_chainId",
        params: &[],
        id: serde_json::Value::Number(1.into()),
        jsonrpc: JSONRPC_VERSION,
    };

    let (status, rpc_response) = send_jsonrpc_request(client, addr, chaind_id, request).await;

    // Verify that HTTP communication was successful
    assert_eq!(status, StatusCode::OK);

    // Verify there was no error in rpc
    assert!(rpc_response.error.is_none());

    // Check chainId
    assert_eq!(rpc_response.result::<String>().unwrap(), expected_id)
}

#[test_context(ServerContext)]
#[tokio::test]
async fn health_check(ctx: &mut ServerContext) {
    let addr = format!("{}/health", ctx.server.public_addr);

    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());

    let request = Request::builder()
        .method(Method::GET)
        .uri(addr)
        .body(Body::default())
        .unwrap();

    let response = client.request(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK)
}
