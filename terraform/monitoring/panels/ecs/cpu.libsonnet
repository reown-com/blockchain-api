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
      title       = 'CPU Utilization',
      datasource  = ds.prometheus,
    )
    .configure(overrides.cpu(defaults.configuration.timeseries_resource))
    .setAlert(alert.new(
      namespace     = 'RPC Proxy',
      name          = "RPC %s - High CPU usage" % vars.environment,
      message       = "RPC %s - High CPU usage" % vars.environment,
      period        = '25m',
      frequency     = '3m',
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
          queryRefId      = 'CPU_Avg',
          queryTimeStart  = '25m',
          reducerType     = 'max',
        ),
      ]
    ))
    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum(rate(cpu_usage_sum[$__rate_interval])) / sum(rate(cpu_usage_count[$__rate_interval]))',
      interval      = '3m',
      legendFormat  = 'CPU Utilization 3m avg.',
      refId         = 'CPU_Avg',
    ))
}
