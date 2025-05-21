use {
    super::{
        check_if_rpc_is_responding_correctly_for_near_protocol,
        check_if_rpc_is_responding_correctly_for_solana,
        check_if_rpc_is_responding_correctly_for_sui,
        check_if_rpc_is_responding_correctly_for_supported_chain,
    },
    crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn pokt_provider_eip155(ctx: &mut ServerContext) {
    let provider = ProviderKind::Pokt;

    // Avax mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:43114",
        "0xa86a",
    )
    .await;

    // Gnosis
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:100", "0x64")
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

    // Binance mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:56", "0x38")
        .await;

    // Ethereum mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:1", "0x1")
        .await;

    // Ethereum holesky
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:17000",
        "0x4268",
    )
    .await;

    // Ethereum Sepolia
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:11155111",
        "0xaa36a7",
    )
    .await;

    // Optimism
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

    // Arbitrum
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

    // Polygon mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:137", "0x89")
        .await;

    // Polygon zkevm
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:1101",
        "0x44d",
    )
    .await;

    // Polygon Amoy
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:80002",
        "0x13882",
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

    // Kaia mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:8217",
        "0x2019",
    )
    .await;

    // zkSync era
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:324", "0x144")
        .await;

    // Scroll
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:534352",
        "0x82750",
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

    // Berachain mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:80094",
        "0x138de",
    )
    .await;

    // Sonic Mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:146", "0x92")
        .await;
}

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn pokt_provider_solana(ctx: &mut ServerContext) {
    let provider = ProviderKind::Pokt;

    // Solana mainnet
    check_if_rpc_is_responding_correctly_for_solana(
        ctx,
        "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
        &provider,
    )
    .await;

    // Legacy non-standart chain id for the mainnet
    check_if_rpc_is_responding_correctly_for_solana(
        ctx,
        "4sgjmw1sunhzsxgspuhpqldx6wiyjntz",
        &provider,
    )
    .await;
}

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn pokt_provider_near(ctx: &mut ServerContext) {
    check_if_rpc_is_responding_correctly_for_near_protocol(ctx, &ProviderKind::Pokt).await;
}

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn pokt_provider_sui(ctx: &mut ServerContext) {
    let provider = ProviderKind::Quicknode;
    // Sui mainnet
    check_if_rpc_is_responding_correctly_for_sui(ctx, &provider, "mainnet", "35834a8a").await;
}
