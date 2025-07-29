local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Cached responses percent by RPC method',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries.withUnit('percent'))

    // Specific target for 'eth_chainId'
    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr          = '100 * (sum(increase(rpc_cached_call_counter_total{method="eth_chainId"}[$__rate_interval])) / on() sum(increase(rpc_call_counter_total[$__rate_interval])))',
      exemplar      = false,
      legendFormat  = 'eth_chainId',
    ))
}
