local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Requests',
      datasource  = ds.cloudwatch,
    )
    .configure(defaults.configuration.timeseries)

    .addTarget(targets.cloudwatch(
      alias       = 'Requests',
      datasource  = ds.cloudwatch,
      namespace   = 'AWS/ApplicationELB',
      metricName  = 'RequestCount',
      dimensions  = {
        LoadBalancer: vars.load_balancer
      },
      matchExact  = true,
      statistic   = 'Sum',
      refId       = 'Requests',
    ))
}
