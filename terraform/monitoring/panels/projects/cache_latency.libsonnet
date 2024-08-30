local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;
local alert          = grafana.alert;
local alertCondition = grafana.alertCondition;

local _configuration = defaults.configuration.timeseries
  .withUnit('ms');

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Cache latency',
      datasource  = ds.prometheus,
    )
    .configure(_configuration)

    .setAlert(vars.environment, alert.new(
      namespace     = 'Blockchain API',
      name          = "%s - ELB High projects registry cache latency" % vars.environment,
      message       = "%s - ELB High projects registry cache latency" % vars.environment,
      period        = '5m',
      frequency     = '1m',
      noDataState   = 'no_data',
      notifications = vars.notifications,
      alertRuleTags = {
        'og_priority': 'P3',
      },
      conditions  = [
        alertCondition.new(
          evaluatorParams = [ 1000 ],
          evaluatorType   = 'gt',
          operatorType    = 'or',
          queryRefId      = 'ProjectsRegistryCacheLatency',
          queryTimeStart  = '5m',
          reducerType     = 'avg',
        ),
      ]
    ))

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum(rate(project_data_local_cache_time_sum[$__rate_interval])) / sum(rate(project_data_local_cache_time_count[$__rate_interval]))',
      refId         = 'ProjectsRegistryCacheLatency',
      legendFormat  = 'Cache',
    ))
}
