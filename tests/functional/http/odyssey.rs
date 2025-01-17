use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain, crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind, test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn odyssey_provider(ctx: &mut ServerContext) {
    // Odyssey
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Odyssey,
        "eip155:911867",
        "0xde9fb",
    )
    .await;
}
