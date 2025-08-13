use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain, crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind, test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn deinfra_provider(ctx: &mut ServerContext) {
    let provider_kind = ProviderKind::DeInfra;

    // DeInfra Mainnet
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider_kind,
        "eip155:100501",
        "0x18895",
    )
    .await;

    // DeInfra Devnet3
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &provider_kind,
        "eip155:1000000003",
        "0x3b9aca03",
    )
    .await;
}
