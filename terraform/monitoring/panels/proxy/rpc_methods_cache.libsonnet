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
    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr          = '(sum by(method) (increase(rpc_cached_call_counter_total{}[$__rate_interval])) / sum(increase(rpc_call_counter_total{status_code="200"}[$__rate_interval]))) * 100',
      exemplar      = false,
      legendFormat  = '__auto',
    )) 
}
