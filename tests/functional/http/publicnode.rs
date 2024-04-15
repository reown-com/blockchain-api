use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain,
    crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn publicnode_provider(ctx: &mut ServerContext) {
    // Ethereum mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Publicnode,
        "eip155:1",
        "0x1",
    )
    .await;

    // Ethereum holesky
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Publicnode,
        "eip155:17000",
        "0x4268",
    )
    .await;

    // Base mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Publicnode,
        "eip155:8453",
        "0x2105",
    )
    .await;

    // BSC mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Publicnode,
        "eip155:56",
        "0x38",
    )
    .await;

    // BSC testnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Publicnode,
        "eip155:97",
        "0x61",
    )
    .await;

    // Avalanche c chain
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Publicnode,
        "eip155:43114",
        "0xa86a",
    )
    .await;

    // Avalanche fuji testnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Publicnode,
        "eip155:43113",
        "0xa869",
    )
    .await;

    // Polygon mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Publicnode,
        "eip155:137",
        "0x89",
    )
    .await;

    // Mantle mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Publicnode,
        "eip155:5000",
        "0x1388",
    )
    .await;
}
