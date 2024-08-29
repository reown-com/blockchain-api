local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels          = grafana.panels;
local targets         = grafana.targets;
local alert           = grafana.alert;
local alertCondition  = grafana.alertCondition;

local _configuration = defaults.configuration.timeseries
  .withUnit('ms');

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Handlers execution duration',
      datasource  = ds.prometheus,
    )
    .configure(_configuration)

    .setAlert(vars.environment, alert.new(
      namespace     = 'Blockchain API',
      name          = "%s - High handlers execution duration" % vars.environment,
      message       = "%s - High handlers execution duration" % vars.environment,
      period        = '5m',
      frequency     = '1m',
      noDataState   = 'no_data',
      notifications = vars.notifications,
      alertRuleTags = {
        'og_priority': 'P3',
      },
      conditions  = [
        alertCondition.new(
          evaluatorParams = [ 3000 ],
          evaluatorType   = 'gt',
          operatorType    = 'or',
          queryRefId      = 'HandlersLatency',
          queryTimeStart  = '5m',
          reducerType     = 'avg',
        ),
      ]
    ))

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum by(task_name) (rate(handler_task_duration_sum[$__rate_interval])) / sum by(task_name) (rate(handler_task_duration_count[$__rate_interval]))',
      exemplar      = false,
      legendFormat  = '__auto',
      refId         = 'HandlersLatency',
    ))
}
