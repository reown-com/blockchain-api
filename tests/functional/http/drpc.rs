use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain, crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind, test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn drpc_provider_evm(ctx: &mut ServerContext) {
    let provider = ProviderKind::Drpc;

    // Ethereum Mainnet
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

    // Ethereum Hoodi
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:560048",
        "0x88bb0",
    )
    .await;

    // Arbitrum One
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:42161",
        "0xa4b1",
    )
    .await;

    // Base
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:8453",
        "0x2105",
    )
    .await;

    // BSC
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:56", "0x38")
        .await;

    // Polygon Mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:137", "0x89")
        .await;

    // Optimism Mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:10", "0xa")
        .await;

    // Unichain Sepolia
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:1301",
        "0x515",
    )
    .await;

    // Kaia mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:8217",
        "0x2019",
    )
    .await;

    // Berachain mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:80094",
        "0x138de",
    )
    .await;

    // Monad testnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:10143",
        "0x279f",
    )
    .await;
}
