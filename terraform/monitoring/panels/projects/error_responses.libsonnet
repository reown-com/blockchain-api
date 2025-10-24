local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;
local alert          = grafana.alert;
local alertCondition = grafana.alertCondition;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Projects registry API responses',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries)

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum by (response)(rate(project_data_requests_total[$__rate_interval]))',
      legendFormat  = '__auto',
    ))
}
