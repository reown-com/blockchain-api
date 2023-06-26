local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels          = grafana.panels;
local targets         = grafana.targets;
local alert           = grafana.alert;
local alertCondition  = grafana.alertCondition;

local error_alert(vars) = alert.new(
  namespace   = 'RPC',
  name        = "RPC %s - Provider Error alert" % vars.environment,
  message     = "RPC %s - Provider Error alert" % vars.environment,
  period      = '5m',
  frequency   = '1m',
  noDataState = 'no_data',
  notifications = vars.notifications,
  alertRuleTags = {
    'og_priority': 'P3',
  },
  
  conditions  = [
    alertCondition.new(
      evaluatorParams = [ 5000 ],
      evaluatorType   = 'gt',
      operatorType    = 'or',
      queryRefId      = 'bad_gateway',
      queryTimeStart  = '5m',
      reducerType     = 'max',
    ),
  ]
);

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Provider Errors',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries)
    .setAlert(error_alert(vars))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'round(sum(increase(http_call_counter{code=\"502\"}[5m])))',
      refId       = "bad_gateway",
    ))
}
