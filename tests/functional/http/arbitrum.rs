use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain, crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind, test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn arbitrum_provider(ctx: &mut ServerContext) {
    let provider_kind = ProviderKind::Arbitrum;

    // Arbitrum One
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider_kind,
        "eip155:42161",
        "0xa4b1",
    )
    .await;

    // Arbitrum Sepolia
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider_kind,
        "eip155:421614",
        "0x66eee",
    )
    .await
}
