local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'JSON-RPC server error code range responses percentage',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries.withUnit('percent'))
    .setAlert(
      vars.environment,
      grafana.alert.new(
        namespace     = vars.namespace,
        name          = "%(env)s - RPC high JSON-RPC server error code responses from provider"  % { env: grafana.utils.strings.capitalize(vars.environment) },
        message       = '%(env)s - RPC high JSON-RPC server error code responses from provider'  % { env: grafana.utils.strings.capitalize(vars.environment) },
        notifications = vars.notifications,
        noDataState   = 'no_data',
        period        = '1m',
        conditions    = [
          grafana.alertCondition.new(
            evaluatorParams = [ 20 ],
            evaluatorType   = 'gt',
            operatorType    = 'or',
            queryRefId      = 'JsonRPCErrorCodesPerCalls',
            queryTimeStart  = '15m',
            queryTimeEnd    = 'now',
            reducerType     = grafana.alert_reducers.Avg
          ),
        ],
      ),
    )

   .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr          = '(sum by(provider) (increase(provider_internal_error_code_counter_total{}[$__rate_interval])) / (sum by(provider) (increase(provider_status_code_counter_total{status_code="200"}[$__rate_interval])) or vector(0))) * 100',
      exemplar      = false,
      legendFormat  = '__auto',
      refId         = 'JsonRPCErrorCodesPerCalls',
    )) 
}
