local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels          = grafana.panels;
local targets         = grafana.targets;
local alert           = grafana.alert;
local alertCondition  = grafana.alertCondition;
local overrides       = defaults.overrides;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Memory Utilization',
      datasource  = ds.prometheus,
    )
    .configure(defaults.overrides.memory(defaults.configuration.timeseries_resource))
    .setAlert(vars.environment, alert.new(
      namespace     = 'Blockchain APi',
      name          = "%s - High Memory (RAM) usage" % vars.environment,
      message       = "%s - High Memory (RAM) usage" % vars.environment,
      period        = '5m',
      frequency     = '1m',
      noDataState   = 'alerting',
      notifications = vars.notifications,
      alertRuleTags = {
        'og_priority': 'P3',
      },
      conditions  = [
        alertCondition.new(
          evaluatorParams = [ 70 ],
          evaluatorType   = 'gt',
          operatorType    = 'or',
          queryRefId      = 'RAM_Avg',
          queryTimeStart  = '5m',
          reducerType     = 'avg',
        ),
      ]
    ))
    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = '(sum(rate(memory_used_sum[$__rate_interval])) / sum(rate(memory_total_sum[$__rate_interval]))) * 100',
      refId       = 'RAM_Avg',
    ))
}
