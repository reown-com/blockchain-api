local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Calls by Chain ID',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries)

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr          = 'sum by(chain_id) (increase(rpc_call_counter_total{}[5m]))',
      exemplar      = false,
      legendFormat  = '__auto',
    ))
}
