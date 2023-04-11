use {
    crate::{context::ServerContext, JSONRPC_VERSION},
    futures_util::{SinkExt, StreamExt},
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_1_mainnet_infura(ctx: &mut ServerContext) {
    let chain_id = "eip155:1";

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
        String::from("\"0x1\"")
    );
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_3_ropsten_infura(ctx: &mut ServerContext) {
    let chain_id = "eip155:3";

    let addr = format!(
        "{}/ws?projectId={}&chainId={}",
        ctx.server.public_addr, ctx.server.project_id, chain_id
    )
    .replace("http", "ws");

    let client = async_tungstenite::tokio::connect_async(addr).await;

    let err = client.err().unwrap();
    if let axum_tungstenite::Error::Http(resp) = err {
        // This chain is no longer supported
        assert_eq!(resp.status(), hyper::StatusCode::GONE);
    } else {
        panic!("Invalid response")
    }
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_42_kovan_infura(ctx: &mut ServerContext) {
    let chain_id = "eip155:42";

    let addr = format!(
        "{}/ws?projectId={}&chainId={}",
        ctx.server.public_addr, ctx.server.project_id, chain_id
    )
    .replace("http", "ws");

    let client = async_tungstenite::tokio::connect_async(addr).await;

    let err = client.err().unwrap();
    if let axum_tungstenite::Error::Http(resp) = err {
        // This chain is no longer supported
        assert_eq!(resp.status(), hyper::StatusCode::GONE);
    } else {
        panic!("Invalid response")
    }
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_4_rinkeby_infura(ctx: &mut ServerContext) {
    let chain_id = "eip155:4";

    let addr = format!(
        "{}/ws?projectId={}&chainId={}",
        ctx.server.public_addr, ctx.server.project_id, chain_id
    )
    .replace("http", "ws");

    let client = async_tungstenite::tokio::connect_async(addr).await;

    let err = client.err().unwrap();
    if let axum_tungstenite::Error::Http(resp) = err {
        // This chain is no longer supported
        assert_eq!(resp.status(), hyper::StatusCode::GONE);
    } else {
        panic!("Invalid response")
    }
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_5_goerli_infura(ctx: &mut ServerContext) {
    let chain_id = "eip155:5";

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
        String::from("\"0x5\"")
    );
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_137_polygon_mainnet_infura(ctx: &mut ServerContext) {
    let chain_id = "eip155:137";

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
        String::from("\"0x89\"")
    );
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_80001_polygon_mumbai_infura(ctx: &mut ServerContext) {
    let chain_id = "eip155:80001";

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
        String::from("\"0x13881\"")
    );
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_10_optimism_mainnet_infura(ctx: &mut ServerContext) {
    let chain_id = "eip155:10";

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
        String::from("\"0xa\"")
    );
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_69_optimism_kovan_infura(ctx: &mut ServerContext) {
    let chain_id = "eip155:69";

    let addr = format!(
        "{}/ws?projectId={}&chainId={}",
        ctx.server.public_addr, ctx.server.project_id, chain_id
    )
    .replace("http", "ws");

    let client = async_tungstenite::tokio::connect_async(addr).await;

    let err = client.err().unwrap();
    if let axum_tungstenite::Error::Http(resp) = err {
        // This chain is no longer supported
        assert_eq!(resp.status(), hyper::StatusCode::GONE);
    } else {
        panic!("Invalid response")
    }
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_420_optimism_goerli_infura(ctx: &mut ServerContext) {
    let chain_id = "eip155:420";

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
        String::from("\"0x1A4\"")
    );
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_42161_arbitrum_mainnet_infura(ctx: &mut ServerContext) {
    let chain_id = "eip155:42161";

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
        String::from("\"0xa4b1\"")
    );
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_421611_arbitrum_rinkeby_infura(ctx: &mut ServerContext) {
    let chain_id = "eip155:421611";

    let addr = format!(
        "{}/ws?projectId={}&chainId={}",
        ctx.server.public_addr, ctx.server.project_id, chain_id
    )
    .replace("http", "ws");

    let client = async_tungstenite::tokio::connect_async(addr).await;

    let err = client.err().unwrap();
    if let axum_tungstenite::Error::Http(resp) = err {
        // This chain is no longer supported
        assert_eq!(resp.status(), hyper::StatusCode::GONE);
    } else {
        panic!("Invalid response")
    }
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_421613_arbitrum_goerli_infura(ctx: &mut ServerContext) {
    let chain_id = "eip155:421613";

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
        String::from("\"0x66eed\"")
    );
}
