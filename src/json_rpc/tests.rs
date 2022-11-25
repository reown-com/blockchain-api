use super::*;

#[test]
fn test_request() {
    let payload: JsonRpcPayload =
        JsonRpcPayload::Request(JsonRpcRequest::new(1.into(), "eth_chainId".into()));

    let serialized = serde_json::to_string(&payload).unwrap();

    assert_eq!(
        &serialized,
        "{\"id\":1,\"jsonrpc\":\"2.0\",\"method\":\"eth_chainId\"}"
    );

    let deserialized: JsonRpcPayload = serde_json::from_str(&serialized).unwrap();

    assert_eq!(&payload, &deserialized)
}

#[test]
fn test_response_result() {
    let payload: JsonRpcPayload =
        JsonRpcPayload::Response(JsonRpcResponse::Result(JsonRpcResult {
            id: 1.into(),
            jsonrpc: Arc::from(JSON_RPC_VERSION),
            result: "some result".into(),
        }));

    let serialized = serde_json::to_string(&payload).unwrap();

    assert_eq!(
        &serialized,
        "{\"id\":1,\"jsonrpc\":\"2.0\",\"result\":\"some result\"}"
    );

    let deserialized: JsonRpcPayload = serde_json::from_str(&serialized).unwrap();

    assert_eq!(&payload, &deserialized)
}

#[test]
fn test_response_error() {
    let payload: JsonRpcPayload = JsonRpcPayload::Response(JsonRpcResponse::Error(JsonRpcError {
        id: 1.into(),
        jsonrpc: Arc::from(JSON_RPC_VERSION),
        error: ErrorResponse {
            code: 32,
            message: Arc::from("some message"),
            data: None,
        },
    }));

    let serialized = serde_json::to_string(&payload).unwrap();

    assert_eq!(
        &serialized,
        "{\"id\":1,\"jsonrpc\":\"2.0\",\"error\":{\"code\":32,\"message\":\"some message\"}}"
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
