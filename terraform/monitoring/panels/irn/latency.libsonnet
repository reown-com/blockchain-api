local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;
local alert           = grafana.alert;
local alertCondition  = grafana.alertCondition;

local _configuration = defaults.configuration.timeseries
  .withUnit('s')
  .withSoftLimit(
    axisSoftMin = 0.4,
    axisSoftMax = 1.5,
  );

local error_alert(vars) = alert.new(
  namespace   = 'Blockchain API',
  name        = "%s - IRN Client latency" % vars.environment,
  message     = "%s - IRN Client latency" % vars.environment,
  period      = '5m',
  frequency   = '1m',
  noDataState = 'alerting',
  notifications = vars.notifications,
  alertRuleTags = {
    'og_priority': 'P3',
  },
  
  conditions  = [
    alertCondition.new(
      evaluatorParams = [ 1.5 ],
      evaluatorType   = 'gt',
      operatorType    = 'or',
      queryRefId      = 'IrnResponseLatency',
      queryTimeStart  = '5m',
      reducerType     = 'avg',
    ),
  ]
);

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Latency',
      datasource  = ds.prometheus,
    )
    .configure(_configuration)

    .setAlert(vars.environment, error_alert(vars))

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum(rate(irn_latency_tracker_sum[$__rate_interval])) / sum(rate(irn_latency_tracker_count[$__rate_interval]))',
      refId         = 'IrnResponseLatency',
      legendFormat  = 'IRN response latency',
    ))
}
