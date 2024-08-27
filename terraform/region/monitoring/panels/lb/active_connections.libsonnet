local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Active Connections',
      datasource  = ds.cloudwatch,
    )
    .configure(defaults.configuration.timeseries)

    .addTarget(targets.cloudwatch(
      datasource      = ds.cloudwatch,
      namespace     = 'AWS/ApplicationELB',
      metricName    = 'ActiveConnectionCount',
      dimensions    = {
        LoadBalancer: vars.load_balancer
      },
      statistic     = 'Average',
    ))
}
