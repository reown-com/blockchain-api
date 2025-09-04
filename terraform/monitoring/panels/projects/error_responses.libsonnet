local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;
local alert          = grafana.alert;
local alertCondition = grafana.alertCondition;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Projects registry API responses',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries)

    .setAlert(vars.environment, alert.new(
      namespace     = 'Blockchain API',
      name          = "%s - Projects registry API unavailability" % vars.environment,
      message       = "%s - Projects registry API unavailability" % vars.environment,
      period        = '5m',
      frequency     = '1m',
      noDataState   = 'no_data',
      notifications = vars.notifications,
      alertRuleTags = {
        'og_priority': 'P3',
      },
      conditions  = [
        alertCondition.new(
          evaluatorParams = [ 1 ],
          evaluatorType   = 'gt',
          operatorType    = 'or',
          queryRefId      = 'ProjectsRegistryUnavailability',
          queryTimeStart  = '1m',
          reducerType     = 'avg',
        ),
      ]
    ))

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum by (response)(rate(project_data_requests_total[$__rate_interval]))',
      legendFormat  = '__auto',
    ))

    // Hidden target for the unavailability alert
    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum (rate(project_data_requests_total{response="registry_temporarily_unavailable"}[$__rate_interval]))',
      refId         = 'ProjectsRegistryUnavailability',
      legendFormat  = '__auto',
      hide          = true,
    ))    
}
