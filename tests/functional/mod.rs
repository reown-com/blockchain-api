use hyper::{http, Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use test_context::test_context;

use crate::context::ServerContext;

mod connection;

#[test_context(ServerContext)]
#[tokio::test]
async fn metrics_check(ctx: &mut ServerContext) {
    let addr = format!("htpps://{}/metrics", ctx.server.private_addr);

    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());

    let request = Request::builder()
        .method(Method::GET)
        .uri(addr)
        .body(Body::default())
        .unwrap();

    let response = client.request(request).await.unwrap();

    assert_eq!(response.status(), http::StatusCode::OK)
}
