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
async fn quicknode_provider(ctx: &mut ServerContext) {
    let provider = ProviderKind::Quicknode;
    // zkSync Mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:324", "0x144")
        .await;
    // Polygon zksync
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:1101",
        "0x44d",
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
    // Berachain Bartio
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider,
        "eip155:80084",
        "0x138d4",
    )
    .await;

    // Optimism Mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, &provider, "eip155:10", "0xa")
        .await;
}

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn quicknode_provider_solana(ctx: &mut ServerContext) {
    let provider = ProviderKind::Quicknode;
    // Solana mainnet
    check_if_rpc_is_responding_correctly_for_solana(
        ctx,
        "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
        &provider,
    )
    .await;

    // Solana devnet
    check_if_rpc_is_responding_correctly_for_solana(
        ctx,
        "EtWTRABZaYq6iMfeYKouRu166VU2xqa1",
        &provider,
    )
    .await;

    // Solana testnet
    check_if_rpc_is_responding_correctly_for_solana(
        ctx,
        "4uhcVJyU9pJkvQyS88uRDiswHXSCkY3z",
        &provider,
    )
    .await;
}

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn quicknode_provider_bitcoin(ctx: &mut ServerContext) {
    let provider = ProviderKind::Quicknode;
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
