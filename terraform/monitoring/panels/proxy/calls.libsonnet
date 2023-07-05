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
      expr          = 'sum by(chain_id) (increase(rpc_call_counter_total{aws_ecs_task_family="%s_rpc-proxy"}[5m]))' % vars.environment,
      exemplar      = false,
      legendFormat  = '__auto',
    ))
}
