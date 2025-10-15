use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain,
    crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn morph_provider(ctx: &mut ServerContext) {
    let provider = ProviderKind::Morph;
    // Morph Mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:2818",
        "0xb02",
    )
    .await;

    // Morph Holesky
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:2810",
        "0xafa",
    )
    .await;
}
