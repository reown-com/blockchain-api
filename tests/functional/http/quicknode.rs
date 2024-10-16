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
async fn quicknode_provider(ctx: &mut ServerContext) {
    // zkSync Mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Quicknode,
        "eip155:324",
        "0x144",
    )
    .await;
    // Unichain Sepolia
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Quicknode,
        "eip155:1301",
        "0x515",
    )
    .await;
    // Berachain Bartio
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Quicknode,
        "eip155:80084",
        "0x138d4",
    )
    .await;
}

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn quicknode_provider_solana(ctx: &mut ServerContext) {
    // Solana mainnet
    check_if_rpc_is_responding_correctly_for_solana(
        ctx,
        "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
        &ProviderKind::Quicknode,
    )
    .await;

    // Solana devnet
    check_if_rpc_is_responding_correctly_for_solana(
        ctx,
        "EtWTRABZaYq6iMfeYKouRu166VU2xqa1",
        &ProviderKind::Quicknode,
    )
    .await;

    // Solana testnet
    check_if_rpc_is_responding_correctly_for_solana(
        ctx,
        "4uhcVJyU9pJkvQyS88uRDiswHXSCkY3z",
        &ProviderKind::Quicknode,
    )
    .await;
}
