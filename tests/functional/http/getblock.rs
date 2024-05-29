use {
    super::{
        check_if_rpc_is_responding_correctly_for_solana,
        check_if_rpc_is_responding_correctly_for_supported_chain,
    },
    crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn getblock_provider_eip155(ctx: &mut ServerContext) {
    let provider = ProviderKind::GetBlock;

    // Ethereum mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:1", "0x1")
        .await;
    // Binance Smart Chain mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:56", "0x38")
        .await;
    // Polygon
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:137", "0x89")
        .await;
    // ZkSync
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:324", "0x144")
        .await;
    // Holesky
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:17000",
        "0x4268",
    )
    .await;
    // Sepolia
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:11155111",
        "0xaa36a7",
    )
    .await;
}

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn getblock_provider_solana(ctx: &mut ServerContext) {
    // Solana mainnet
    check_if_rpc_is_responding_correctly_for_solana(
        ctx,
        "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
        &ProviderKind::GetBlock,
    )
    .await;
}
