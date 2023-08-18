use {
    crate::{context::ServerContext, JSONRPC_VERSION},
    futures_util::{SinkExt, StreamExt},
};

pub(crate) mod infura;
pub(crate) mod zora;

async fn check_if_rpc_is_responding_correctly_for_supported_chain(
    ctx: &ServerContext,
    chain_id: &str,
    expected_id: &str,
) {
    let addr = format!(
        "{}/ws?projectId={}&chainId={}",
        ctx.server.public_addr, ctx.server.project_id, chain_id
    )
    .replace("http", "ws");

    let (client, _) = async_tungstenite::tokio::connect_async(addr).await.unwrap();
    let request = jsonrpc::Request {
        method: "eth_chainId",
        params: &[],
        id: serde_json::Value::Number(1.into()),
        jsonrpc: JSONRPC_VERSION,
    };

    let (mut tx, mut rx) = client.split();

    tx.send(axum_tungstenite::Message::Text(
        serde_json::to_string(&request).unwrap(),
    ))
    .await
    .unwrap();

    let response = rx.next().await.unwrap().unwrap();
    let response: jsonrpc::Response = serde_json::from_str(&response.to_string()).unwrap();

    assert!(response.error.is_none());
    assert_eq!(
        response.result.unwrap().to_string(),
        format!("\"{expected_id}\"")
    );
}
