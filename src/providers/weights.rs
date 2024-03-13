use {
    super::{ProviderKind, WeightResolver},
    crate::env::ChainId,
    prometheus_http_query::response::PromqlResult,
    std::collections::HashMap,
    tracing::log::warn,
};

/// The amount of successful and failed requests to a provider
///
/// Availability(success_counter, failure_counter)
#[derive(Debug, Copy, Clone)]
pub struct Availability(u64, u64);

pub type ParsedWeights = HashMap<ProviderKind, (HashMap<ChainId, Availability>, Availability)>;

#[tracing::instrument(skip_all)]
pub fn parse_weights(prometheus_data: PromqlResult) -> ParsedWeights {
    let mut weights_data = HashMap::new();
    // fill weights with pair of ProviderKind -> HashMap<ChainId, Availability>
    prometheus_data.data().as_vector().iter().for_each(|v| {
        for metrics in v.iter() {
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

            let provider_kind = match ProviderKind::from_str(&provider) {
                Some(provider_kind) => provider_kind,
                None => {
                    warn!("Failed to parse provider kind in metric: {}", provider);
                    continue;
                }
            };

            let amount = metrics.sample().value();

            let (provider_map, provider_availability) = weights_data
                .entry(provider_kind)
                .or_insert_with(|| (HashMap::new(), Availability(0, 0)));

            let chain_availability = provider_map
                .entry(chain_id)
                .or_insert_with(|| Availability(0, 0));

            if status_code.starts_with('2') || status_code == "404" || status_code == "400" {
                provider_availability.0 += amount as u64;
                chain_availability.0 += amount as u64;
            } else {
                provider_availability.1 += amount as u64;
                chain_availability.1 += amount as u64;
            }
        }
    });
    weights_data
}

const PERFECT_RATIO: f64 = 1.0;

#[tracing::instrument]
fn calculate_chain_weight(
    provider_availability: Availability,
    chain_availability: Availability,
) -> u64 {
    let Availability(provider_success, provider_failure) = provider_availability;

    // Sum failed and successful calls for provider
    let Some(provider_failures_squared) = provider_failure.checked_mul(provider_failure) else {
        // 1 is minimal value for chain weight
        return 0;
    };

    let provider_total = provider_success + provider_failures_squared;

    let Availability(chain_success, chain_failure) = chain_availability;

    // Sum failed and successful calls for chain
    let Some(chain_failures_squared) = chain_failure.checked_mul(chain_failure) else {
        // 1 is minimal value for chain weight
        return 0;
    };

    let chain_total = chain_success + chain_failures_squared;

    // Provider success rate is the amount of successful calls to provider over the
    // total amount of calls to provider
    let provider_success_rate = if provider_total == 0 {
        // If chain had no calls, implicitely had no issues, so we assume it's fine
        PERFECT_RATIO
    } else {
        provider_success as f64 / provider_total as f64
    };

    // Chain success rate is the amount of successful calls to chain over the total
    // amount of calls to chain within one provider
    let chain_success_rate = if chain_total == 0 {
        PERFECT_RATIO
    } else {
        chain_success as f64 / chain_total as f64
    };

    // As success rate is always a float within (0,1> range
    // multiplying it by 100_00 will result in a number between (0,100_00>
    // representing percentage up to two decimal places using u32
    // (e.g. 99.99% = 99_99) which allows for usage of atomic operations This means
    // that provider scales linearly, but chain scales exponentially (each chain
    // fail also is counted as provider fail)
    let weight = provider_success_rate * chain_success_rate * 10000.0;
    weight as u64
}

#[tracing::instrument(skip_all)]
pub fn update_values(weight_resolver: &WeightResolver, parsed_weights: ParsedWeights) {
    for (provider, (chain_availabilities, provider_availability)) in parsed_weights {
        for (chain_id, chain_availability) in chain_availabilities {
            let chain_id = chain_id.0;
            let chain_weight = calculate_chain_weight(chain_availability, provider_availability);

            let Some(provider_chain_weight) = weight_resolver.get(&chain_id) else {
                warn!(
                    "Chain {} not found in weight resolver: {:?}",
                    chain_id, weight_resolver
                );
                continue;
            };

            let Some(weight) = provider_chain_weight.get(&provider) else {
                warn!(
                    "Weight for {} not found in weight map: {:?}",
                    &provider, provider_chain_weight
                );
                continue;
            };

            weight.update_value(chain_weight);
        }
    }
}

pub fn record_values(weight_resolver: &WeightResolver, metrics: &crate::Metrics) {
    for (chain_id, provider_chain_weight) in weight_resolver {
        for (provider_kind, weight) in provider_chain_weight {
            let weight = weight.value();
            metrics.record_provider_weight(provider_kind, chain_id.to_owned(), weight)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn calcaulate_weights() {
        // The chain in this provider has 75% success rate
        let chain_availability = super::Availability(75, 25);
        // The provider has around 71.42% success rate
        let provider_availability = super::Availability(125, 50);

        let weight = super::calculate_chain_weight(chain_availability, provider_availability);

        assert_eq!(weight, 51);
    }

    #[test]
    fn calcaulate_weights_with_unused_chain() {
        // The chain in this provider has 100% success rate (as per our assumption
        // because it hasnt failed yet)
        let chain_availability = super::Availability(0, 0);
        // The provider has around 71.42% success rate
        let provider_availability = super::Availability(125, 50);

        let weight = super::calculate_chain_weight(chain_availability, provider_availability);

        assert_eq!(weight, 476);
    }

    #[test]
    fn calcaulate_weights_with_unused_provider() {
        // The chain in this provider has 100% success rate (as per our assumption
        // because it hasnt failed yet)
        let chain_availability = super::Availability(0, 0);
        // The provider has 100% success rate (as per our assumption
        // because it hasnt failed yet)
        let provider_availabilities = super::Availability(0, 0);

        let weight = super::calculate_chain_weight(chain_availability, provider_availabilities);

        // 100% * 100% = 100%
        assert_eq!(weight, 10_000);
    }
}
