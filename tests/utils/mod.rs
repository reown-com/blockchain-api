use {
    axum::{
        body::{to_bytes, Body},
        http::{HeaderValue, StatusCode},
    },
    sqlx::{postgres::PgPoolOptions, PgPool},
    std::env,
};

const RESPONSE_MAX_BYTES: usize = 512 * 1024; // 512 KB

pub async fn send_jsonrpc_request(
    base_addr: String,
    chain: &str,
    rpc_request: jsonrpc::Request<'static>,
) -> (StatusCode, jsonrpc::Response) {
    let addr = base_addr + chain;

    let json = serde_json::to_string(&rpc_request).unwrap();
    let client = reqwest::Client::new();
    let response = client
        .post(addr.clone())
        .header("Content-Type", "application/json")
        .body(json.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.headers().get("Content-Type"),
        Some(&HeaderValue::from_static("application/json"))
    );

    let status = response.status();
    let bytes = response.bytes().await.unwrap();
    let body = Body::from(bytes);
    let body = to_bytes(body, RESPONSE_MAX_BYTES).await.unwrap();
    (
        status,
        serde_json::from_slice(&body).unwrap_or_else(|_| {
            panic!(
                "Failed to parse response '{:?}' ({} / {:?})",
                &body, &addr, &json
            )
        }),
    )
}

pub async fn get_postgres_pool() -> PgPool {
    let postgres = PgPoolOptions::new()
        .connect(&env::var("RPC_PROXY_POSTGRES_URI").unwrap())
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&postgres).await.unwrap();
    postgres
}
