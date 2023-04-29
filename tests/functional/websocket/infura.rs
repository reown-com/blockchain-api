use {
    super::{
        check_if_rpc_is_responding_correctly_for_decomissioned,
        check_if_rpc_is_responding_correctly_for_supported_chain,
    },
    crate::context::ServerContext,
    test_context::test_context,
};

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_1_mainnet_infura(ctx: &mut ServerContext) {
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:1", "0x1").await
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_3_ropsten_infura(ctx: &mut ServerContext) {
    check_if_rpc_is_responding_correctly_for_decomissioned(ctx, "eip155:3").await
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_42_kovan_infura(ctx: &mut ServerContext) {
    check_if_rpc_is_responding_correctly_for_decomissioned(ctx, "eip155:42").await
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_4_rinkeby_infura(ctx: &mut ServerContext) {
    check_if_rpc_is_responding_correctly_for_decomissioned(ctx, "eip155:4").await
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_5_goerli_infura(ctx: &mut ServerContext) {
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:5", "0x5").await
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_10_optimism_mainnet_infura(ctx: &mut ServerContext) {
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:10", "0xa").await
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_69_optimism_kovan_infura(ctx: &mut ServerContext) {
    check_if_rpc_is_responding_correctly_for_decomissioned(ctx, "eip155:69").await
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_420_optimism_goerli_infura(ctx: &mut ServerContext) {
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:420", "0x1A4").await
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_42161_arbitrum_mainnet_infura(ctx: &mut ServerContext) {
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:42161", "0xa4b1").await
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_421611_arbitrum_rinkeby_infura(ctx: &mut ServerContext) {
    check_if_rpc_is_responding_correctly_for_decomissioned(ctx, "eip155:421611").await
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_421613_arbitrum_goerli_infura(ctx: &mut ServerContext) {
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:421613", "0x66eed").await
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_1313161554_aurora_mainnet_infura(ctx: &mut ServerContext) {
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:1313161554", "0x4e454152")
        .await
}

#[test_context(ServerContext)]
#[tokio::test]
async fn eip155_1313161554_aurora_testnet_infura(ctx: &mut ServerContext) {
    check_if_rpc_is_responding_correctly_for_supported_chain(ctx, "eip155:1313161555", "0x4e454153")
        .await
}
