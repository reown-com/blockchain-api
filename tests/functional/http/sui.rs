use {
    super::check_if_rpc_is_responding_correctly_for_sui, crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind, test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn sui_provider(ctx: &mut ServerContext) {
    let provider = ProviderKind::Sui;
    // Sui mainnet
    check_if_rpc_is_responding_correctly_for_sui(ctx, &provider, "mainnet", "35834a8a").await;
    // Sui testnet
    check_if_rpc_is_responding_correctly_for_sui(ctx, &provider, "testnet", "4c78adac").await;
    // Sui devnet
    check_if_rpc_is_responding_correctly_for_sui(ctx, &provider, "devnet", "6ee96fc3").await;
}
