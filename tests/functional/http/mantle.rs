use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain,
    crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn mantle_provider(ctx: &mut ServerContext) {
    // Mantle mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Mantle,
        "eip155:5000",
        "0x1388",
    )
    .await;
    // Mantle testnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Mantle,
        "eip155:5001",
        "0x1389",
    )
    .await;
}
