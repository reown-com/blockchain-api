local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels         = grafana.panels;
local targets        = grafana.targets;
local alert          = grafana.alert;
local alertCondition = grafana.alertCondition;

local no_data_alert(vars) = alert.new(
  namespace   = 'RPC',
  name        = "RPC %s - No Data alert" % vars.environment,
  message     = "RPC %s - No Data alert" % vars.environment,
  period      = '5m',
  frequency   = '1m',
  noDataState = 'alerting',
  notifications = vars.notifications,
  alertRuleTags = {
    'og_priority': 'P3',
  },
  
  conditions  = [
    alertCondition.new(
      evaluatorParams = [ 1 ],
      evaluatorType   = 'lt',
      operatorType    = 'or',
      queryRefId      = 'total',
      queryTimeStart  = '5m',
      reducerType     = 'sum',
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
    .setAlert(no_data_alert(vars))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(http_call_counter{aws_ecs_task_family=\"%s_rpc-proxy\",code=~\"5.+\"}[5m])) or vector(0)' % vars.environment,
      refId       = "errors",
      exemplar    = false,
      hide        = true,
    ))
    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(http_call_counter{aws_ecs_task_family=\"%s_rpc-proxy\",code=\"429\"}[5m])) or vector(0)' % vars.environment,
      refId       = 'rate_limits',
      exemplar    = false,
      hide        = true,
    ))
    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(http_call_counter{aws_ecs_task_family=\"%s_rpc-proxy\"}[5m]))' % vars.environment,
      refId       = 'total',
      exemplar    = false,
      hide        = true,
    ))
    .addTarget(targets.math(
      expr        = '(1 - (($errors + $rate_limits) / $total)) * 100',
      refId       = "Availability",
    ))
}
