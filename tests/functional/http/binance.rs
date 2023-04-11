use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain,
    crate::context::ServerContext,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_56_bsc_mainnet(ctx: &mut ServerContext) {
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:56", "0x38").await
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_97_bsc_testnet(ctx: &mut ServerContext) {
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:97", "0x61").await
}
