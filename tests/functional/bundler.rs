use rpc_proxy::{
    providers::mock_alto::MockAltoUrls, test_helpers::spawn_blockchain_api_with_params,
};
use serde_json::json;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
#[ignore]
async fn default_bundler() {
    let bundler_server = MockServer::start().await;

    let response = ResponseTemplate::new(200).set_body_json(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": "0x1"
    }));

    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_partial_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_getUserOperationReceipt",
            "params": null
        })))
        .respond_with(response)
        .mount(&bundler_server)
        .await;

    // tracing_subscriber::fmt()
    //     .with_env_filter(
    //         EnvFilter::builder()
    //             .with_default_directive(LevelFilter::ERROR.into())
    //             .parse("DEBUG")
    //             .expect("Invalid log level"),
    //     )
    //     .with_span_events(FmtSpan::CLOSE)
    //     .with_ansi(false)
    //     .init();

    let server_url = spawn_blockchain_api_with_params(rpc_proxy::test_helpers::Params {
        validate_project_id: false,
        override_bundler_urls: Some(MockAltoUrls {
            bundler_url: bundler_server.uri().parse().unwrap(),
            paymaster_url: bundler_server.uri().parse().unwrap(),
        }),
    })
    .await;
    let mut url = server_url.join("/v1/bundler").unwrap();
    url.query_pairs_mut()
        .append_pair("projectId", "test")
        .append_pair("chainId", "eip155:1");

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .json(&jsonrpc::Request {
            method: "eth_getUserOperationReceipt",
            params: None,
            id: serde_json::Value::Number(1.into()),
            jsonrpc: Some("2.0"),
        })
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(
        response
            .json::<serde_json::Value>()
            .await
            .unwrap()
            .get("result")
            .unwrap()
            .as_str(),
        Some("0x1")
    );

    let mut url = server_url.join("/v1/bundler").unwrap();
    url.query_pairs_mut()
        .append_pair("projectId", "test")
        .append_pair("chainId", "eip155:1")
        .append_pair("bundler", "pimlico");

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .json(&jsonrpc::Request {
            method: "eth_getUserOperationReceipt",
            params: None,
            id: serde_json::Value::Number(1.into()),
            jsonrpc: Some("2.0"),
        })
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(
        response
            .json::<serde_json::Value>()
            .await
            .unwrap()
            .get("result")
            .unwrap()
            .as_str(),
        Some("0x1")
    );
}

#[tokio::test]
#[ignore]
async fn bundler_url() {
    let bundler_server = MockServer::start().await;

    let response = ResponseTemplate::new(200).set_body_json(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": "0x1"
    }));

    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_partial_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_getUserOperationReceipt",
            "params": null
        })))
        .respond_with(response)
        .mount(&bundler_server)
        .await;

    // tracing_subscriber::fmt()
    //     .with_env_filter(
    //         EnvFilter::builder()
    //             .with_default_directive(LevelFilter::ERROR.into())
    //             .parse("DEBUG")
    //             .expect("Invalid log level"),
    //     )
    //     .with_span_events(FmtSpan::CLOSE)
    //     .with_ansi(false)
    //     .init();

    let url = spawn_blockchain_api_with_params(rpc_proxy::test_helpers::Params {
        validate_project_id: false,
        override_bundler_urls: None,
    })
    .await;
    let mut url = url.join("/v1/bundler").unwrap();
    url.query_pairs_mut()
        .append_pair("projectId", "test")
        .append_pair("chainId", "eip155:1")
        .append_pair("bundler", &bundler_server.uri());

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .json(&jsonrpc::Request {
            method: "eth_getUserOperationReceipt",
            params: None,
            id: serde_json::Value::Number(1.into()),
            jsonrpc: Some("2.0"),
        })
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(
        response
            .json::<serde_json::Value>()
            .await
            .unwrap()
            .get("result")
            .unwrap()
            .as_str(),
        Some("0x1")
    );
}
