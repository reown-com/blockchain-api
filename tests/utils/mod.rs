use {
    hyper::{body, client::HttpConnector, Body, Client, Method, Request, StatusCode},
    hyper_tls::HttpsConnector,
};

pub async fn send_jsonrpc_request(
    client: Client<HttpsConnector<HttpConnector>>,
    base_addr: String,
    chain: &str,
    rpc_request: jsonrpc::Request<'static>,
) -> (StatusCode, jsonrpc::Response) {
    let addr = base_addr + chain;

    let json = serde_json::to_string(&rpc_request).unwrap();
    let req_body = Body::from(json.clone());

    let request = Request::builder()
        .method(Method::POST)
        .uri(addr.clone())
        .header("Content-Type", "application/json")
        .body(req_body)
        .unwrap();

    let response = client.request(request).await.unwrap();

    let (parts, body) = response.into_parts();
    let body = body::to_bytes(body).await.unwrap();
    (
        parts.status,
        serde_json::from_slice(&body).unwrap_or_else(|_| {
            panic!(
                "Failed to parse response '{:?}' ({} / {:?})",
                &body, &addr, &json
            )
        }),
    )
}
