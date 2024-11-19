local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars, provider)::
    panels.timeseries(
      title       = provider,
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries)
    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum by(status_code) (increase(provider_status_code_counter_total{provider="%s"}[$__rate_interval]))' % provider,
      legendFormat  = '__auto',
    ))

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum by(status_code) (increase(provider_status_code_counter_total{provider="%s", status_code=~"401|402|403"}[$__rate_interval]))' % provider,
      refId         = 'Provider4xxErrors',
      hide          = true,
    ))
    .setAlert(
      vars.environment,
      grafana.alert.new(
        namespace     = vars.namespace,
        name          = "%(env)s - Provider 40[1-3] Errors alert" % { env: grafana.utils.strings.capitalize(vars.environment) },
        message       = '%(env)s - Provider 40[1-3] Errors alert' % { env: grafana.utils.strings.capitalize(vars.environment) },
        notifications = vars.notifications,
        noDataState   = 'no_data',
        period        = '0m',
        conditions    = [
          grafana.alertCondition.new(
            evaluatorParams = [ 0 ],
            evaluatorType   = 'gt',
            operatorType    = 'or',
            queryRefId      = 'Provider4xxErrors',
            queryTimeStart  = '15m',
            queryTimeEnd    = 'now',
            reducerType     = grafana.alert_reducers.Avg
          ),
        ],
      ),
    )
}
