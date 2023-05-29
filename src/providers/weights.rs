use {
    super::{ProviderKind, WeightResolver},
    crate::env::ChainId,
    prometheus_http_query::response::PromqlResult,
    std::collections::HashMap,
    tracing::log::warn,
};

#[derive(Debug)]
pub struct Availability(u32, u32);

pub type ParsedWeights = HashMap<String, (HashMap<ChainId, Availability>, Availability)>;

pub fn parse_weights(prometheus_data: PromqlResult) -> ParsedWeights {
    let mut weights_data = HashMap::new();
    // fill weights with pair of ProviderKind -> HashMap<ChainId, Availability>
    prometheus_data.data().as_vector().iter().for_each(|v| {
        for metrics in v.into_iter() {
            let mut metric = metrics.metric().to_owned();
            let chain_id = if let Some(chain_id) = metric.remove("chain_id") {
                ChainId(chain_id)
            } else {
                warn!("No chain_id found in metric: {:?}", metric);
                continue;
            };

            let Some(status_code) = metric.remove("status_code") else { 
                warn!("No status_code found in metric: {:?}", metric);
                continue;
            };

            let Some(provider) = metric.remove("provider") else {
                warn!("No provider found in metric: {:?}", metric);
                continue;
            };
            let amount = metrics.sample().value();

            let (provider_map, provider_availability) = weights_data
                .entry(provider)
                .or_insert_with(|| (HashMap::new(), Availability(0, 0)));

            let chain_availability = provider_map
                .entry(chain_id)
                .or_insert_with(|| Availability(0, 0));

            if status_code == "200" {
                provider_availability.0 += amount as u32;
                chain_availability.0 += amount as u32;
            } else {
                provider_availability.1 += amount as u32;
                chain_availability.1 += amount as u32;
            }
        }
    });
    weights_data
}

fn calculate_chain_weight(
    provider_availability: Availability,
    chain_availability: &Availability,
) -> u32 {
    let Availability(provider_success, provider_failure) = provider_availability;
    let provider_total = provider_success + provider_failure;

    // If chain had no calls, implicitely had no issues, so we assume it's fine
    if provider_total == 0 {
        return 10000;
    }

    let Availability(chain_success, chain_failure) = chain_availability;
    let chain_total = chain_success + chain_failure;

    let provider_success_rate = provider_success as f64 / provider_total as f64;
    let chain_success_rate = *chain_success as f64 / chain_total as f64;

    // This means that provider scales linearly, but chain scales exponentially
    // (each chain fail also is counted as provider fail)
    let weight = provider_success_rate * chain_success_rate * 10000.0;
    weight as u32
}

pub fn update_values(weight_resolver: &WeightResolver, parsed_weights: ParsedWeights) {
    for (provider, (chain_availabilities, provider_availability)) in parsed_weights {
        for (chain_id, chain_availability) in chain_availabilities {
            let chain_id = chain_id.0;
            let chain_weight = calculate_chain_weight(chain_availability, &provider_availability);

            let Some(provider_chain_weight) = weight_resolver.get(&chain_id) else {
                warn!("Chain {} not found in weight resolver: {:?}", chain_id, weight_resolver);
                continue;
            };

            let Some(atomic) = provider_chain_weight
                .get(&ProviderKind::from_str(&provider).unwrap()) else {
                    warn!("Weight for {} not found in weight map: {:?}", &provider, provider_chain_weight);
                    continue;
                };

            atomic
                .0
                .store(chain_weight, std::sync::atomic::Ordering::SeqCst);
        }
    }
}
