use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain, crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind, test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn infura_provider(ctx: &mut ServerContext) {
    let provider = ProviderKind::Infura;
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

    // Polgyon mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:137", "0x89")
        .await;

    // Optimism mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:10", "0xa")
        .await;

    // Optimism Sepolia
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:11155420",
        "0xaa37dc",
    )
    .await;

    // Arbitrum mainnet
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

    // Celo
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:42220",
        "0xa4ec",
    )
    .await;

    // Linea Mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:59144",
        "0xe708",
    )
    .await;

    // Mantle Mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:5000",
        "0x1388",
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

    // Base Sepolia
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:84532",
        "0x14a34",
    )
    .await;
}
