local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels          = grafana.panels;
local targets         = grafana.targets;
local alert           = grafana.alert;
local alertCondition  = grafana.alertCondition;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Rate limited entries count',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries)

    .setAlert(vars.environment, alert.new(
      namespace     = 'Blockchain API',
      name          = "%s - High rate-limiting entries count" % vars.environment,
      message       = "%s - High rate-limiting entries count" % vars.environment,
      period        = '15m',
      frequency     = '5m',
      noDataState   = 'alerting',
      notifications = vars.notifications,
      alertRuleTags = {
        'og_priority': 'P3',
      },
      conditions  = [
        alertCondition.new(
          evaluatorParams = [ 250 ],
          evaluatorType   = 'gt',
          operatorType    = 'or',
          queryRefId      = 'Rate_limited_count',
          queryTimeStart  = '5m',
          reducerType     = 'avg',
        ),
      ]
    ))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr          = 'max(rate(rate_limited_entries_sum{}[$__rate_interval]) / rate(rate_limited_entries_count{}[$__rate_interval]))',
      legendFormat  = 'app in-memory entries',
      refId         = 'Rate_limited_count',
    ))
}
