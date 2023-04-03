#[cfg(feature = "test-localhost")]
#[test_context(ServerContext)]
#[tokio::test]
async fn metrics_check(ctx: &mut ServerContext) {
    let addr = format!("https://{}/metrics", ctx.server.private_addr.unwrap());

    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());

    let request = Request::builder()
        .method(Method::GET)
        .uri(addr)
        .body(Body::default())
        .unwrap();

    let response = client.request(request).await.unwrap();

    assert_eq!(response.status(), http::StatusCode::OK)
}
