use {
    super::{ProviderKind, WeightResolver},
    crate::env::ChainId,
    prometheus_http_query::response::PromqlResult,
    std::collections::HashMap,
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
            let chain_id = ChainId(metric.remove("chain_id").unwrap());
            let status_code = metric.remove("status_code").unwrap();
            let provider = metric.remove("provider").unwrap();
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
    // If provider total is 0, then chain for that specific provider is 0 as well
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

fn calculate_provider_weight(availability: Availability) -> u32 {
    let Availability(success, failure) = availability;
    let total = success + failure;
    if total == 0 {
        return 10000;
    }
    let success_rate = success as f64 / total as f64;
    (success_rate * 10000.0) as u32
}

pub fn update_values(weight_resolver: &WeightResolver, parsed_weights: ParsedWeights) {
    for (provider, (chain_availabilities, provider_availability)) in parsed_weights {
        // let provider_kind = provider.parse::<ProviderKind>().unwrap();
        // let provider_weight = calculate_provider_weight(provider_availability);
        for (chain_id, chain_availability) in chain_availabilities {
            let chain_id = chain_id.0;
            let chain_weight = calculate_chain_weight(chain_availability, &provider_availability);

            let provider_chain_weight = weight_resolver.get(&chain_id).unwrap();

            let atomic = provider_chain_weight
                .get(&ProviderKind::from_str(&provider).unwrap())
                .unwrap();

            atomic
                .0
                .store(chain_weight, std::sync::atomic::Ordering::SeqCst);

            // let provider_chain_weight = weight_resolver
            //     .entry(chain_id)
            //     .or_insert_with(|| Vec::new());
            // provider_chain_weight.push((provider_kind,
            // Weight(provider_weight))); let provider_chain_weight
            // = weight_resolver     .entry(provider)
            //     .or_insert_with(|| Vec::new());
            // provider_chain_weight.push((provider_kind,
            // Weight(chain_weight)));
        }
    }
}

// I've got to getme

// 50 - 150
// 1/4

// Get availability for provider
// Get availability for chain per provider

// Vec<ChainId, Vec<Provider, Availability>>
// HashMap<ProviderKind, HashMap<ChainId, Availability>>
