local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels         = grafana.panels;
local targets        = grafana.targets;
local alert          = grafana.alert;
local alertCondition = grafana.alertCondition;

local error_alert(vars) = alert.new(
  namespace   = 'RPC',
  name        = "RPC %s - Availability" % vars.environment,
  message     = "RPC %s - Availability" % vars.environment,
  period      = '5m',
  frequency   = '1m',
  noDataState = 'alerting',
  notifications = vars.notifications,
  alertRuleTags = {
    'og_priority': 'P3',
  },
  
  conditions  = [
    alertCondition.new(
      evaluatorParams = [ 95 ],
      evaluatorType   = 'lt',
      operatorType    = 'or',
      queryRefId      = 'availability',
      queryTimeStart  = '5m',
      reducerType     = 'avg',
    ),
  ]
);

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Availability',
      datasource  = ds.prometheus,
    )
    .configure(
      defaults.configuration.timeseries
        .withUnit('percent')
        .withSoftLimit(
          axisSoftMin = 98,
          axisSoftMax = 100,
        )
    )
    .setAlert(error_alert(vars))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = '(1-(sum(rate(http_call_counter_total{aws_ecs_task_family="%s_rpc-proxy",code=~"5.+"}[5m])) or vector(0))/(sum(rate(http_call_counter_total{aws_ecs_task_family="%s_rpc-proxy"}[5m]))))*100' % [vars.environment, vars.environment],
      refId       = "availability",
      exemplar    = false,
    ))
}
