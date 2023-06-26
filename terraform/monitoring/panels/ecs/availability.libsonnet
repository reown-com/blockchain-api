local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels         = grafana.panels;
local targets        = grafana.targets;
local alert          = grafana.alert;
local alertCondition = grafana.alertCondition;

local error_alert(vars) = alert.new(
  namespace = "RPC",
  name      = "RPC %s - Availability" % vars.environment,
  message   = "RPC %s - Availability" % vars.environment,
  period    = "5m",
  frequency = "1m",
  noDataState = "no_data",
  notifications = vars.notifications,
  alertRuleTags = {
    'og_priority': 'P3',
  },

  conditions = [
    alertCondition.new(
      evaluatorParams = [ 99.5 ],
      evaluatorType   = 'lt',
      operatorType    = 'or',
      queryRefId      = 'Availability',
      queryTimeStart  = '5m',
      reducerType     = 'avg',
    )
  ]
);


{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Availability',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries.withUnit('percent'))
    .setAlert(error_alert(vars))

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
