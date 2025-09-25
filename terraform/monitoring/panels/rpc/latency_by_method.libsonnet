local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

local _configuration = defaults.configuration.timeseries
  .withUnit('s');

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Methods latency',
      datasource  = ds.prometheus,
    )
    .configure(_configuration)

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum by(method) (rate(json_rpc_call_latency_tracker_sum[$__rate_interval])) / sum by(method) (rate(json_rpc_call_latency_tracker_count[$__rate_interval]))',
      exemplar      = false,
      legendFormat  = '__auto',
    ))
}
