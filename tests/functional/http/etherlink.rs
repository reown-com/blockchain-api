use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain,
    rpc_proxy::providers::ProviderKind,
    crate::context::ServerContext,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn etherlink_provider_eip155_42793(ctx: &mut ServerContext) {
    // Etherlink Mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Etherlink,
        "eip155:42793",
        "0xa729",
    )
    .await;

    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Etherlink,
        "eip155:128123",
        "0x1f47b",
    )
    .await;
}