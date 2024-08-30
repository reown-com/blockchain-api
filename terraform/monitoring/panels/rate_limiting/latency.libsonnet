local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels         = grafana.panels;
local targets        = grafana.targets;
local alert          = grafana.alert;
local alertCondition = grafana.alertCondition;

local _configuration = defaults.configuration.timeseries
  .withUnit('ms');

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Rate limiter latency',
      datasource  = ds.prometheus,
    )
    .configure(_configuration)

    .setAlert(vars.environment, alert.new(
      namespace     = 'Blockchain API',
      name          = "%s - Rate limiter high latency" % vars.environment,
      message       = "%s - Rate limiter high latency" % vars.environment,
      period        = '5m',
      frequency     = '1m',
      noDataState   = 'no_data',
      notifications = vars.notifications,
      alertRuleTags = {
        'og_priority': 'P3',
      },
      conditions  = [
        alertCondition.new(
          evaluatorParams = [ 100 ],
          evaluatorType   = 'gt',
          operatorType    = 'or',
          queryRefId      = 'RateLimiterLatency',
          queryTimeStart  = '5m',
          reducerType     = 'avg',
        ),
      ]
    ))

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum(rate(rate_limiting_latency_tracker_sum[$__rate_interval])) / sum(rate(rate_limiting_latency_tracker_count[$__rate_interval]))',
      refId         = 'RateLimiterLatency',
      legendFormat  = 'Latency',
    ))
}
