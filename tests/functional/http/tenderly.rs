use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain, crate::context::ServerContext,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
async fn tenderly_provider(ctx: &mut ServerContext) {
    // Ethereum Mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:1", "0x1").await;

    // Görli
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:5", "0x5").await;

    // Sepolia
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:11155111", "0xaa36a7")
        .await;

    // Optimism Mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:10", "0xa").await;

    // Optimism Görli
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:420", "0x1A4").await;

    // Polygon Mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:137", "0x89").await;

    // Polygon Mumbai
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:80001", "0x13881").await;

    // Base Mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:8453", "0x2105").await;

    // Base Görli
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:84531", "0x14a33").await;
}
