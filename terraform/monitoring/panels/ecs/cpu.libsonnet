local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;
local overrides = defaults.overrides;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'CPU Utilization',
      datasource  = ds.cloudwatch,
    )
    .configure(overrides.cpu(defaults.configuration.timeseries_resource))
    .setAlert(defaults.alerts.cpu(
      namespace     = 'RPC Proxy',
      env           = vars.environment,
      title         = 'ECS',
      notifications = vars.notifications,
    ))

    .addTarget(targets.cloudwatch(
      alias       = 'CPU (Max)',
      datasource  = ds.cloudwatch,
      dimensions  = {
        ServiceName: vars.ecs_service_name
      },
      metricName  = 'CPUUtilization',
      namespace   = 'AWS/ECS',
      statistic   = 'Maximum',
      refId       = 'CPU_Max',
    ))
    .addTarget(targets.cloudwatch(
      alias       = 'CPU (Avg)',
      datasource  = ds.cloudwatch,
      dimensions  = {
        ServiceName: vars.ecs_service_name
      },
      metricName  = 'CPUUtilization',
      namespace   = 'AWS/ECS',
      statistic   = 'Average',
      refId       = 'CPU_Avg',
    ))
}
