use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain,
    crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn base_provider_eip155_8453_and_84531(ctx: &mut ServerContext) {
    // Base mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Base,
        "eip155:8453",
        "0x2105",
    )
    .await;

    // Base Sepolia
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Base,
        "eip155:84532",
        "0x14a34",
    )
    .await
}
