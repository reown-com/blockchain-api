local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels         = grafana.panels;
local targets        = grafana.targets;
local alert          = grafana.alert;
local alertCondition = grafana.alertCondition;

{
  new(ds, vars, provider, alertPeriod)::
    panels.timeseries(
      title       = provider,
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries)

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr          = 'sum by(chain_id) (increase(provider_status_code_counter_total{provider="%s"}[$__rate_interval]))' % provider,
      legendFormat  = '__auto',
    ))

    // Hidden target for the provider availability alert

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr          = '(sum(increase(provider_status_code_counter_total{provider="%s", status_code="200"}[$__rate_interval])) / sum(increase(provider_status_code_counter_total{provider="%s"}[$__rate_interval]))) * 100' % [provider, provider],
      legendFormat  = '__auto',
      exemplar    = false,
      refId       = 'providerAvailabilityPercent',
      hide        = true,
    ))

    .setAlert(vars.environment, alert.new(
      namespace     = 'Blockchain API',
      name          = "%s - Provider availability drop" % vars.environment,
      message       = "%s - Provider availability drop" % vars.environment,
      period        = alertPeriod,
      frequency     = '1m',
      noDataState   = 'no_data',
      notifications = vars.notifications,
      alertRuleTags = {
        'og_priority': 'P3',
      },
      conditions  = [
        alertCondition.new(
          evaluatorParams = [ 90 ],
          evaluatorType   = 'lt',
          operatorType    = 'or',
          queryRefId      = 'providerAvailabilityPercent',
          queryTimeStart  = alertPeriod,
          reducerType     = 'avg',
        ),
      ]
    ))
}
