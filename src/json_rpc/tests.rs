use super::*;

#[test]
fn test_request_serialized() {
    assert_eq!(
        &serde_json::to_string(&JsonRpcPayload::Request(JsonRpcRequest::new(
            "1".into(),
            "eth_chainId".into()
        )))
        .unwrap(),
        "{\"id\":\"1\",\"jsonrpc\":\"2.0\",\"method\":\"eth_chainId\",\"params\":null}"
    );
}

#[test]
fn test_request_deserialized() {
    assert_eq!(
        &serde_json::from_str::<JsonRpcRequest>(
            "{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"eth_chainId\",\"params\":null}"
        )
        .unwrap(),
        &JsonRpcRequest::new(1.into(), "eth_chainId".into(),),
    );
    assert_eq!(
        &serde_json::from_str::<JsonRpcRequest>(
            "{\"id\":\"abc\",\"jsonrpc\":\"2.0\",\"method\":\"eth_chainId\",\"params\":null}"
        )
        .unwrap(),
        &JsonRpcRequest::new("abc".into(), "eth_chainId".into(),),
    );
}

#[test]
fn test_response_result() {
    let payload: JsonRpcPayload =
        JsonRpcPayload::Response(JsonRpcResponse::Result(JsonRpcResult {
            id: "1".into(),
            jsonrpc: JSON_RPC_VERSION.clone(),
            result: "some result".into(),
        }));

    let serialized = serde_json::to_string(&payload).unwrap();

    assert_eq!(
        &serialized,
        "{\"id\":\"1\",\"jsonrpc\":\"2.0\",\"result\":\"some result\"}"
    );

    let deserialized: JsonRpcPayload = serde_json::from_str(&serialized).unwrap();

    assert_eq!(&payload, &deserialized)
}

#[test]
fn test_response_error() {
    let payload: JsonRpcPayload = JsonRpcPayload::Response(JsonRpcResponse::Error(JsonRpcError {
        id: 1.into(),
        jsonrpc: JSON_RPC_VERSION.clone(),
        error: ErrorResponse {
            code: 32,
            message: Arc::from("some message"),
            data: None,
        },
    }));

    let serialized = serde_json::to_string(&payload).unwrap();

    assert_eq!(
        &serialized,
        "{\"id\":1,\"jsonrpc\":\"2.0\",\"error\":{\"code\":32,\"message\":\"some message\",\"data\":null}}"
    );

    let deserialized: JsonRpcPayload = serde_json::from_str(&serialized).unwrap();

    assert_eq!(&payload, &deserialized)
}

#[test]
fn test_deserialize_iridium_method() {
    let serialized = "{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"iridium_subscription\",\"params\"\
                      :{\"id\":\"test_id\",\"data\":{\"topic\":\"test_topic\",\"message\":\"\
                      test_message\"}}}";
    assert!(serde_json::from_str::<'_, JsonRpcPayload>(serialized).is_ok());
}
