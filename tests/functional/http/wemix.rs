use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain, crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind, test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn wemix_provider(ctx: &mut ServerContext) {
    // Wemix Mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Wemix,
        "eip155:1111",
        "0x457",
    )
    .await;

    // Wemix Testnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Wemix,
        "eip155:1112",
        "0x458",
    )
    .await;
}
