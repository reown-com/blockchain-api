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
async fn lava_provider_evm(ctx: &mut ServerContext) {
    let provider = ProviderKind::Lava;
    // Ethereum mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:1", "0x1")
        .await;

    // Ethereum Sepolia
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:11155111",
        "0xaa36a7",
    )
    .await;

    // Ethereum Holesky
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:17000",
        "0x4268",
    )
    .await;

    // Arbitrum Mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:42161",
        "0xa4b1",
    )
    .await;

    // Arbitrum Sepolia
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:421614",
        "0x66eee",
    )
    .await;

    // Base Mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:8453",
        "0x2105",
    )
    .await;
}

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn lava_provider_solana(ctx: &mut ServerContext) {
    let provider = ProviderKind::Lava;
    // Solana Mainnet
    check_if_rpc_is_responding_correctly_for_solana(
        ctx,
        "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
        &provider,
    )
    .await;
}
