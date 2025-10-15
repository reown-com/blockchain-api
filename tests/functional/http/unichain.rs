use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain,
    crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn unichain_provider_eip155_1301(ctx: &mut ServerContext) {
    // Unichain Sepolia
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Unichain,
        "eip155:1301",
        "0x515",
    )
    .await;
}
