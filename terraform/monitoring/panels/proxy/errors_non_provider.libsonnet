local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels          = grafana.panels;
local targets         = grafana.targets;
local alert           = grafana.alert;
local alertCondition  = grafana.alertCondition;

local error_alert(vars) = alert.new(
  namespace   = 'RPC',
  name        = "RPC %s - Non-Provider Error alert" % vars.environment,
  message     = "RPC %s - Non-Provider Error alert" % vars.environment,
  period      = '5m',
  frequency   = '1m',
  notifications = vars.notifications,
  alertRuleTags = {
    'og_priority': 'P3',
  },
  
  conditions  = [
    alertCondition.new(
      evaluatorParams = [ 10 ],
      evaluatorType   = 'gt',
      operatorType    = 'or',
      queryRefId      = 'Availability',
      queryTimeStart  = '5m',
      reducerType     = 'sum',
    ),
  ]
);

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Non-Provider Errors',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries)
    .setAlert(error_alert(vars))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'round(sum(increase(http_call_counter{code=~\"5.+\"}[5m])))',
      refId       = "total",
      exemplar    = true,
      hide        = true,
    ))
    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'round(sum(increase(http_call_counter{code=\"502\"}[5m])))',
      refId       = "bad_gateway",
      exemplar    = true,
      hide        = true,
    ))
    .addTarget(targets.math(
      expr        = '$total - $bad_gateway',
      refId       = "Availability",
    ))
}
