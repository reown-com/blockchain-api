local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Cache latency',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries)

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr          = 'sum(rate(non_rpc_providers_cache_latency_tracker_sum[$__rate_interval])) / sum(rate(non_rpc_providers_cache_latency_tracker_count[$__rate_interval]))',
      legendFormat  = '__auto',
    ))
}
