use {
    super::{
        check_if_rpc_is_responding_correctly_for_decomissioned,
        check_if_rpc_is_responding_correctly_for_supported_chain,
    },
    crate::context::ServerContext,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
async fn infura_provider(ctx: &mut ServerContext) {
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:1", "0x1").await;

    check_if_rpc_is_responding_correctly_for_decomissioned(ctx, "eip155:3").await;

    // Kovan - decomissioned
    check_if_rpc_is_responding_correctly_for_decomissioned(ctx, "eip155:42").await;

    // Rinkeby - decomissioned
    check_if_rpc_is_responding_correctly_for_decomissioned(ctx, "eip155:4").await;

    // Goerli mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:5", "0x5").await;

    // Polgyon mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:137", "0x89").await;

    // Polygon mumbai
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:80001", "0x13881").await;

    // Optimism mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:10", "0xa").await;

    // Optimism kovan - decomissioned
    check_if_rpc_is_responding_correctly_for_decomissioned(ctx, "eip155:69").await;

    // Optimism goerli
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:420", "0x1A4").await;

    // Arbitrum mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:42161", "0xa4b1").await;

    // Arbitrum rinkeby - decomissioned
    check_if_rpc_is_responding_correctly_for_decomissioned(ctx, "eip155:421611").await;

    // Arbitrum goerli
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:421613", "0x66eed").await;

    // Aurora mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        "eip155:1313161554",
        "0x4e454152",
    )
    .await;

    // Aurora testnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:1313161555", "0x4e454153")
        .await
}
