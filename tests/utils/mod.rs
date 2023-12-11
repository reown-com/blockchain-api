use {
    axum::http::HeaderValue,
    hyper::{body, client::HttpConnector, Body, Client, Method, Request, StatusCode},
    hyper_tls::HttpsConnector,
    rand::{distributions::Alphanumeric, Rng},
    sqlx::{postgres::PgPoolOptions, PgPool},
    std::env,
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
    assert_eq!(
        response.headers().get("Content-Type"),
        Some(&HeaderValue::from_static("application/json"))
    );

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

pub async fn get_postgres_pool() -> PgPool {
    let postgres = PgPoolOptions::new()
        .connect(&env::var("RPC_PROXY_POSTGRES_URI").unwrap())
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&postgres).await.unwrap();
    postgres
}

pub fn generate_random_string(len: usize) -> String {
    let rng = rand::thread_rng();
    rng.sample_iter(&Alphanumeric)
        .filter_map(|b| {
            let c = b as char;
            if c.is_ascii_alphanumeric() && c.is_ascii_lowercase() || c.is_ascii_digit() {
                Some(c)
            } else {
                None
            }
        })
        .take(len)
        .collect()
}
