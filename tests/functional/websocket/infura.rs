use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain,
    crate::context::ServerContext,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn infura_provider_websocket(ctx: &mut ServerContext) {
    // Ethereum mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:1", "0x1").await;

    // Optimism mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:10", "0xa").await;

    // Optimism Sepolia
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:11155420", "0xaa37dc")
        .await;

    // Arbitrum mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:42161", "0xa4b1").await;

    // Arbitrum Sepolia
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:421614", "0x66eee").await;
}
