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
          queryRefId      = 'HandlersLatency',
          queryTimeStart  = '5m',
          reducerType     = 'avg',
        ),
        alertCondition.new(
          evaluatorParams = [ 10000 ],
          evaluatorType   = 'gt',
          queryRefId      = 'TransactionsHandlerLatency',
          queryTimeStart  = '5m',
          reducerType     = 'avg',
        ),
      ]
    ))

    // Distinguishing between all handlers and transactions list for the different
    // alerting threshold since Solana transactions list taking longer then others

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum by(name) (rate(future_duration_sum{name!="transactions", future_name="handler_task"}[$__rate_interval])) / sum by(name) (rate(future_duration_count{name!="transactions", future_name="handler_task"}[$__rate_interval]))',
      exemplar      = false,
      legendFormat  = '__auto',
      refId         = 'HandlersLatency',
    ))

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum by(name) (rate(future_duration_sum{name="transactions", future_name="handler_task"}[$__rate_interval])) / sum by(name) (rate(future_duration_count{name="transactions", future_name="handler_task"}[$__rate_interval]))',
      exemplar      = false,
      legendFormat  = '__auto',
      refId         = 'TransactionsHandlerLatency',
    ))
}
