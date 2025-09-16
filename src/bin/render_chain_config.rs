use rpc_proxy::chain_config::ACTIVE_CONFIG;

fn main() {
    let json = serde_json::to_string_pretty(&*ACTIVE_CONFIG).unwrap();
    std::fs::write("terraform/monitoring/chain_config.json", json).unwrap();
}
