use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain,
    crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn zksync_provider_eip155_324_and_280(ctx: &mut ServerContext) {
    // ZkSync mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::ZKSync,
        "eip155:324",
        "0x144",
    )
    .await;

    // ZkSync Sepolia testnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::ZKSync,
        "eip155:300",
        "0x12c",
    )
    .await
}
