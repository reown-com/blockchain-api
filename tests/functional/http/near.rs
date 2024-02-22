use {
    super::check_if_rpc_is_responding_correctly_for_near_protocol,
    crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn near_provider(ctx: &mut ServerContext) {
    check_if_rpc_is_responding_correctly_for_near_protocol(ctx, &ProviderKind::Near).await;
}
