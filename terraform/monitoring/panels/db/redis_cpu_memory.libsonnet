local grafana         = import '../../grafonnet-lib/grafana.libsonnet';
local defaults        = import '../../grafonnet-lib/defaults.libsonnet';

local panels          = grafana.panels;
local targets         = grafana.targets;
local alert           = grafana.alert;
local alertCondition  = grafana.alertCondition;

//.configure(defaults.cpu_overrides(defaults.configuration.timeseries_resource))
local _configuration  = defaults.overrides.cpu_memory(defaults.configuration.timeseries_resource);

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Redis CPU/Memory EU',
      datasource  = ds.cloudwatch,
    )
    .configure(defaults.overrides.cpu_memory(defaults.configuration.timeseries_resource))

    .setAlert(defaults.alerts.cpu_mem(
      namespace     = 'RPC Proxy',
      env           = vars.environment,
      notifications = vars.notifications,
      priority      = 'P2',
    ))

    .addTarget(targets.cloudwatch(
      alias       = 'CPU (Max)',
      datasource  = ds.cloudwatch,
      dimensions  = {
        CacheClusterId : vars.redis_cluster_id
      },
      matchExact  = true,
      metricName  = 'CPUUtilization',
      namespace   = 'AWS/ElastiCache',
      statistic   = 'Maximum',
      refId       = 'CPU_Max',
    ))
    .addTarget(targets.cloudwatch(
      alias       = 'CPU (Avg)',
      datasource  = ds.cloudwatch,
      dimensions  = {
        CacheClusterId : vars.redis_cluster_id
      },
      matchExact  = true,
      metricName  = 'CPUUtilization',
      namespace   = 'AWS/ElastiCache',
      statistic   = 'Average',
      refId       = 'CPU_Avg',
    ))

    .addTarget(targets.cloudwatch(
      alias       = 'Memory (Max)',
      datasource  = ds.cloudwatch,
      dimensions  = {
        CacheClusterId : vars.redis_cluster_id
      },
      matchExact  = true,
      metricName  = 'DatabaseMemoryUsagePercentage',
      namespace   = 'AWS/ElastiCache',
      statistic   = 'Maximum',
      refId       = 'Mem_Max',
    ))
    .addTarget(targets.cloudwatch(
      alias       = 'Memory (Avg)',
      datasource  = ds.cloudwatch,
      dimensions  = {
        CacheClusterId : vars.redis_cluster_id
      },
      matchExact  = true,
      metricName  = 'DatabaseMemoryUsagePercentage',
      namespace   = 'AWS/ElastiCache',
      statistic   = 'Average',
      refId       = 'Mem_Avg',
    ))
}
