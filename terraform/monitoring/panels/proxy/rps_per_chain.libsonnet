local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'RPS by Chain ID',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries.withUnit('reqps'))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr          = 'sum by(chain_id) (rate(rpc_call_counter_total{}[$__rate_interval]))',
      exemplar      = false,
      legendFormat  = '__auto',
    ))
}
