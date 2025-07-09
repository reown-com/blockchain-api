{
    new(ds, vars, row, pos):: [
        row.new("Chain RPC Router Overview"),
        // Overview panels for all chains
        (import 'chain_availability.libsonnet').new(ds, vars) { gridPos: pos._4 },
        (import 'chain_latency.libsonnet').new(ds, vars) { gridPos: pos._4 },
        (import 'chain_requests.libsonnet').new(ds, vars) { gridPos: pos._4 },
        (import 'chain_retries.libsonnet').new(ds, vars) { gridPos: pos._4 },
    ] + [
        row.new('%s - %s ' % [chain.name, chain.caip2], collapsed=true)
        .addPanels([
            (import 'chain_providers_availability.libsonnet').new(ds, vars, chain) { gridPos: pos._4 },
            (import 'chain_providers_latency.libsonnet').new(ds, vars, chain) { gridPos: pos._4 },
            (import 'chain_providers_requests.libsonnet').new(ds, vars, chain) { gridPos: pos._4 },
            (import 'chain_providers_distribution.libsonnet').new(ds, vars, chain) { gridPos: pos._4 }
        ])
        for chain in vars.chain_config.chains
    ]
}

// TODO per-method breakdown (+routing later)
// TODO archive vs not breakdown (+routing later)
