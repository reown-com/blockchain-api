use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain,
    crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn aurora_provider(ctx: &mut ServerContext) {
    // Aurora Mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Aurora,
        "eip155:1313161554",
        "0x4e454152",
    )
    .await;

    // Aurora Testnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Aurora,
        "eip155:1313161555",
        "0x4e454153",
    )
    .await
}
