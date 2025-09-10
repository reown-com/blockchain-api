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
      expr          = 'sum by(future_name) (rate(future_duration_count[$__rate_interval]))',
      legendFormat  = "{{future_name}}"
    ))
}
