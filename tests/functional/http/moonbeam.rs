use {
    super::check_if_rpc_is_responding_correctly_for_supported_chain,
    crate::context::ServerContext,
    rpc_proxy::providers::ProviderKind,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
#[ignore]
async fn moonbeam_provider_eip155_1284(ctx: &mut ServerContext) {
    // Moonbeam GLMR
    check_if_rpc_is_responding_correctly_for_supported_chain(
        ctx,
        &ProviderKind::Moonbeam,
        "eip155:1284",
        "0x504",
    )
    .await;
}
