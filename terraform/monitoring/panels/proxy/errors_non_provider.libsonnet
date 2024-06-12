local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels          = grafana.panels;
local targets         = grafana.targets;
local alert           = grafana.alert;
local alertCondition  = grafana.alertCondition;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Non-Provider Errors',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries)
    .setAlert(
      vars.environment,
      grafana.alert.new(
        namespace     = vars.namespace,
        name          = "%(env)s - Non-Provider Errors alert"     % { env: grafana.utils.strings.capitalize(vars.environment) },
        message       = '%(env)s - Non-Provider Errors alert'  % { env: grafana.utils.strings.capitalize(vars.environment) },
        notifications = vars.notifications,
        noDataState   = 'no_data',
        period        = '0m',
        conditions    = [
          grafana.alertCondition.new(
            evaluatorParams = [ 0 ],
            evaluatorType   = 'gt',
            operatorType    = 'or',
            queryRefId      = 'NonProviderErrors',
            queryTimeStart  = '15m',
            queryTimeEnd    = 'now',
            reducerType     = grafana.alert_reducers.Avg
          ),
        ],
      ),
    )

    .addTarget(targets.prometheus(
      datasource   = ds.prometheus,
      expr         = 'sum by(code) (rate(http_call_counter_total{code=~"5[0-9][0-24-9]"}[$__rate_interval]))',
      refId        = "NonProviderErrors",
      exemplar     = true,
      legendFormat = '__auto',
    )) 
}
