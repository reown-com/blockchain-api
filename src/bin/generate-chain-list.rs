use rpc_proxy::env;
use rpc_proxy::providers::Weight;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

fn supported_chains() -> Vec<HashMap<String, (String, Weight)>> {
    vec![
        env::binance::supported_chains(),
        env::infura::supported_chains(),
        env::omnia::supported_chains(),
        env::pokt::supported_chains(),
        env::publicnode::supported_chains(),
        env::zksync::supported_chains(),
    ]
}

// fn ws_supported_chains() -> Vec<HashMap<String, (String, Weight)>> {
//     vec![env::infura::ws_supported_chains()]
// }

const EIP155_PREFIX: &str = "eip155:";

#[tokio::main]
async fn main() {
    for chain_id in supported_chains()
        .iter()
        .map(|entry| entry.keys())
        .flatten()
        .collect::<HashSet<_>>()
    {
        if chain_id.starts_with(EIP155_PREFIX) {
            let eip155_id = &chain_id[EIP155_PREFIX.len()..];

            let chain_name = get_chain_name(eip155_id).await;

            println!("{chain_name}");
        }
    }
}

async fn get_chain_name(eip155_id: &str) -> String {
    let url = format!(
        "https://raw.githubusercontent.com/ethereum-lists/chains/master/_data/chains/eip155-{eip155_id}.json"
    );
    let json = reqwest::get(&url)
        .await
        .expect(&format!("expected successful get on URL {url}"))
        .json::<Value>()
        .await
        .expect("valid JSON");

    json["name"]
        .as_str()
        .expect(&format!(
            "Expected string value in `name` field on URL {url}"
        ))
        .to_owned()
}
