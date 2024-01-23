use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain,
    crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn infura_provider(ctx: &mut ServerContext) {
    // Ethereum mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Infura,
        "eip155:1",
        "0x1",
    )
    .await;

    // Ethereum Goerli
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Infura,
        "eip155:5",
        "0x5",
    )
    .await;

    // Ethereum Sepolia
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Infura,
        "eip155:11155111",
        "0xaa36a7",
    )
    .await;

    // Polgyon mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Infura,
        "eip155:137",
        "0x89",
    )
    .await;

    // Polygon mumbai
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Infura,
        "eip155:80001",
        "0x13881",
    )
    .await;

    // Optimism mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Infura,
        "eip155:10",
        "0xa",
    )
    .await;

    // Optimism goerli
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Infura,
        "eip155:420",
        "0x1A4",
    )
    .await;

    // Arbitrum mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Infura,
        "eip155:42161",
        "0xa4b1",
    )
    .await;

    // Arbitrum goerli
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Infura,
        "eip155:421613",
        "0x66eed",
    )
    .await;

    // Celo
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Infura,
        "eip155:42220",
        "0xa4ec",
    )
    .await;

    // Base Goerli
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Infura,
        "eip155:84531",
        "0x14a33",
    )
    .await
}
