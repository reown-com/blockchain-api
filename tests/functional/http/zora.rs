use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain,
    crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn zora_provider_eip155_7777777_and_999(ctx: &mut ServerContext) {
    // Zora mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Zora,
        "eip155:7777777",
        "0x76adf1",
    )
    .await;

    // Zora Sepolia
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Zora,
        "eip155:999999999",
        "0x3b9ac9ff",
    )
    .await
}
