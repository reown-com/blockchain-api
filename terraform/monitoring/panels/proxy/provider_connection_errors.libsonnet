local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels          = grafana.panels;
local targets         = grafana.targets;
local alert           = grafana.alert;
local alertCondition  = grafana.alertCondition;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Provider connection errors',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries)
    .setAlert(
      vars.environment,
      grafana.alert.new(
        namespace     = vars.namespace,
        name          = "%(env)s - RPC provider connection errors"  % { env: grafana.utils.strings.capitalize(vars.environment) },
        message       = '%(env)s - RPC provider connection errors'  % { env: grafana.utils.strings.capitalize(vars.environment) },
        notifications = vars.notifications,
        noDataState   = 'no_data',
        period        = '5m',
        conditions    = [
          grafana.alertCondition.new(
            evaluatorParams = [ 10 ],
            evaluatorType   = 'gt',
            operatorType    = 'or',
            queryRefId      = 'ProviderConnErrors',
            queryTimeStart  = '15m',
            queryTimeEnd    = 'now',
            reducerType     = grafana.alert_reducers.Avg
          ),
        ],
      ),
    )

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr          = 'sum by(provider) (increase(provider_connection_error_counter_total{}[$__rate_interval]))',
      exemplar      = false,
      legendFormat  = '__auto',
      refId         = "ProviderConnErrors",
    ))
}
