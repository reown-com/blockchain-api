use {
    super::check_if_rpc_is_responding_correctly_for_solana, crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind, test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn drpc_provider_solana(ctx: &mut ServerContext) {
    let provider = ProviderKind::Drpc;

    // Solana mainnet
    check_if_rpc_is_responding_correctly_for_solana(
        ctx,
        "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
        &provider,
    )
    .await;
}
