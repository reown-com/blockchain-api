use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain,
    crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn allnodes_provider(ctx: &mut ServerContext) {
    let provider_kind = ProviderKind::Allnodes;

    // Ethereum Mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider_kind,
        "eip155:1",
        "0x1",
    )
    .await;
}
