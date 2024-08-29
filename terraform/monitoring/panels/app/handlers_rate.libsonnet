local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Handlers rate',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries.withUnit('reqps'))

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum by(task_name) (rate(handler_task_duration_count[$__rate_interval]))',
      legendFormat  = "{{task_name}}"
    ))
}
