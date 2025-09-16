local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Retries',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries.withUnit('retries/min'))

    .addTargets([
      // Retry count for each chain
      targets.prometheus(
        datasource    = ds.prometheus,
        expr          = 'sum(rate(rpc_call_retries_sum{chain_id="%s"}[$__rate_interval]))' % chain.caip2,
        exemplar      = false,
        legendFormat  = chain.name,
        refId         = "ChainRetries%s" % chain.caip2,
      )
      for chain in vars.chain_config.chains
    ])
} 