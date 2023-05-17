use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain,
    crate::context::ServerContext,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
async fn binance_provider(ctx: &mut ServerContext) {
    // Binance mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:56", "0x38").await;

    // Binance testnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:97", "0x61").await
}
