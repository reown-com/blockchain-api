use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain, crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind, test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn edexa_provider(ctx: &mut ServerContext) {
    // tests/functional/http/edexa.rs Mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Edexa,
        "eip155:5424",
        "0x1530",
    )
    .await;

    // edeXa Testnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Edexa,
        "eip155:1995",
        "0x7cb",
    )
    .await
}
