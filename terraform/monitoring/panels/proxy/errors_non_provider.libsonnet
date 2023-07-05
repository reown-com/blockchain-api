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
  noDataState = 'no_data',
  notifications = vars.notifications,
  alertRuleTags = {
    'og_priority': 'P3',
  },
  
  conditions  = [
    alertCondition.new(
      evaluatorParams = [ 15 ],
      evaluatorType   = 'gt',
      operatorType    = 'or',
      queryRefId      = 'non_provider_errors',
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
      expr        = 'round(sum(increase(http_call_counter_total{code=~"50[0-1]|503|50[5-9]|5[1-9][0-9]"}[5m])))',
      refId       = "non_provider_errors",
      exemplar    = true,
    )) 
}
