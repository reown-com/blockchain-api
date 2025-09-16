local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

local _configuration = defaults.configuration.timeseries
  .withUnit('s')
  .withSoftLimit(
    axisSoftMin = 0,
    axisSoftMax = 0.2,
  );

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Latency',
      datasource  = ds.prometheus,
    )
    .configure(_configuration)

    .addTargets([
      // Total average latency across all providers for each chain (including retries)
      targets.prometheus(
        datasource    = ds.prometheus,
        expr          = 'sum(rate(chain_latency_tracker_sum{chain_id="%s"}[$__rate_interval])) / sum(rate(chain_latency_tracker_count{chain_id="%s"}[$__rate_interval]))' % [chain.caip2, chain.caip2],
        exemplar      = false,
        legendFormat  = chain.name,
        refId         = "ChainLatency%s" % chain.caip2,
      )
      for chain in vars.chain_config.chains
    ])
} 