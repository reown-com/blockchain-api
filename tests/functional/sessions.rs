use rpc_proxy::test_helpers::spawn_blockchain_api_with_params;
use serde_json::json;

#[tokio::test]
#[ignore]
async fn test_sessions_create_v1_format() {
    let server_url = spawn_blockchain_api_with_params(rpc_proxy::test_helpers::Params {
        validate_project_id: false,
        ..Default::default()
    })
    .await;

    let url = server_url
        .join("/v1/sessions/eip155:1:0x1234567890123456789012345678901234567890?projectId=test")
        .unwrap();

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .json(&json!({
            "expiry": 1234567890,
            "signer": {
                "type": "keys",
                "data": {}
            },
            "permissions": [],
            "policies": []
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let json: serde_json::Value = response.json().await.unwrap();
    let public_key = json["key"]["publicKey"].as_str().unwrap();

    // v1 format: ASCII-hex encoded (no 0x prefix)
    assert!(!public_key.starts_with("0x"));
    // Should be ASCII-hex encoded (all characters are hex digits)
    assert!(public_key.chars().all(|c| c.is_ascii_hexdigit()));
    // Length should be double the original hex string (65 bytes * 2 * 2 = 260)
    assert_eq!(public_key.len(), 260);
}

#[tokio::test]
#[ignore]
async fn test_sessions_create_v2_format() {
    let server_url = spawn_blockchain_api_with_params(rpc_proxy::test_helpers::Params {
        validate_project_id: false,
        ..Default::default()
    })
    .await;

    let url = server_url
        .join("/v1/sessions/eip155:1:0x1234567890123456789012345678901234567890?projectId=test&v=2")
        .unwrap();

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .json(&json!({
            "expiry": 1234567890,
            "signer": {
                "type": "keys",
                "data": {}
            },
            "permissions": [],
            "policies": []
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let json: serde_json::Value = response.json().await.unwrap();
    let public_key = json["key"]["publicKey"].as_str().unwrap();

    // v2 format: Direct hex with 0x prefix
    assert!(public_key.starts_with("0x"));
    // Should be a valid hex string after 0x
    assert!(public_key[2..].chars().all(|c| c.is_ascii_hexdigit()));
    // Uncompressed public key should be 65 bytes = 130 hex chars + 2 for "0x"
    assert_eq!(public_key.len(), 132);
}

#[tokio::test]
#[ignore]
async fn test_sessions_create_invalid_version_defaults_to_v1() {
    let server_url = spawn_blockchain_api_with_params(rpc_proxy::test_helpers::Params {
        validate_project_id: false,
        ..Default::default()
    })
    .await;

    let url = server_url
        .join("/v1/sessions/eip155:1:0x1234567890123456789012345678901234567890?projectId=test&v=99")
        .unwrap();

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .json(&json!({
            "expiry": 1234567890,
            "signer": {
                "type": "keys",
                "data": {}
            },
            "permissions": [],
            "policies": []
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let json: serde_json::Value = response.json().await.unwrap();
    let public_key = json["key"]["publicKey"].as_str().unwrap();

    // Should default to v1 format (no 0x prefix)
    assert!(!public_key.starts_with("0x"));
    assert_eq!(public_key.len(), 260);
}

#[tokio::test]
#[ignore]
async fn test_sessions_create_version_backward_compatibility() {
    let server_url = spawn_blockchain_api_with_params(rpc_proxy::test_helpers::Params {
        validate_project_id: false,
        ..Default::default()
    })
    .await;

    let client = reqwest::Client::new();

    // Test without version parameter (should default to v1)
    let url_no_version = server_url
        .join("/v1/sessions/eip155:1:0x1234567890123456789012345678901234567890?projectId=test")
        .unwrap();

    let response_no_version = client
        .post(url_no_version)
        .json(&json!({
            "expiry": 1234567890,
            "signer": {
                "type": "keys",
                "data": {}
            },
            "permissions": [],
            "policies": []
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response_no_version.status(), reqwest::StatusCode::OK);
    let json_no_version: serde_json::Value = response_no_version.json().await.unwrap();
    let public_key_no_version = json_no_version["key"]["publicKey"].as_str().unwrap();

    // Test with v=1 (should be same as no version)
    let url_v1 = server_url
        .join("/v1/sessions/eip155:1:0x1234567890123456789012345678901234567890?projectId=test&v=1")
        .unwrap();

    let response_v1 = client
        .post(url_v1)
        .json(&json!({
            "expiry": 1234567890,
            "signer": {
                "type": "keys",
                "data": {}
            },
            "permissions": [],
            "policies": []
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response_v1.status(), reqwest::StatusCode::OK);
    let json_v1: serde_json::Value = response_v1.json().await.unwrap();
    let public_key_v1 = json_v1["key"]["publicKey"].as_str().unwrap();

    // Both should have the same format (ASCII-hex encoded)
    assert_eq!(public_key_no_version.len(), public_key_v1.len());
    assert!(!public_key_no_version.starts_with("0x"));
    assert!(!public_key_v1.starts_with("0x"));
}