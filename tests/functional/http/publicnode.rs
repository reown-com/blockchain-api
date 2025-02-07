use {
    super::{
        check_if_rpc_is_responding_correctly_for_bitcoin,
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
async fn publicnode_provider(ctx: &mut ServerContext) {
    let provider = ProviderKind::Publicnode;

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

    // Ethereum holesky
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:17000",
        "0x4268",
    )
    .await;

    // Base mainnet
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

    // BSC mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:56", "0x38")
        .await;

    // BSC testnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:97", "0x61")
        .await;

    // Avalanche c chain
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:43114",
        "0xa86a",
    )
    .await;

    // Avalanche fuji testnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:43113",
        "0xa869",
    )
    .await;

    // Polygon mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:137", "0x89")
        .await;

    // Polygon amoy testnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:80002",
        "0x13882",
    )
    .await;

    // Mantle mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:5000",
        "0x1388",
    )
    .await;

    // Sei
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:1329",
        "0x531",
    )
    .await;

    // Scroll
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:534352",
        "0x82750",
    )
    .await;

    // Scroll Sepolia
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:534351",
        "0x8274f",
    )
    .await;

    // Gnosis
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:100", "0x64")
        .await;

    // Optimism mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:10", "0xa")
        .await;

    // Arbitrum One
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:42161",
        "0xa4b1",
    )
    .await;

    // Berachain Bartio
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:80084",
        "0x138d4",
    )
    .await;

    // Unichain Sepolia
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:1301",
        "0x515",
    )
    .await;
}

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn publicnode_provider_bitcoin(ctx: &mut ServerContext) {
    let provider = ProviderKind::Publicnode;

    // Bitcoin mainnet
    check_if_rpc_is_responding_correctly_for_bitcoin(
        ctx,
        "000000000019d6689c085ae165831e93",
        &provider,
    )
    .await;

    // Bitcoin testnet
    check_if_rpc_is_responding_correctly_for_bitcoin(
        ctx,
        "000000000933ea01ad0ee984209779ba",
        &provider,
    )
    .await;
}

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn quicknode_provider_solana(ctx: &mut ServerContext) {
    let provider = ProviderKind::Publicnode;
    // Solana mainnet
    check_if_rpc_is_responding_correctly_for_solana(
        ctx,
        "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
        &provider,
    )
    .await;
}
