local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

local _alert(namespace, env, notifications) = grafana.alert.new(
  namespace     = namespace,
  name          = "%(env)s - 5XX alert"     % { env: grafana.utils.strings.capitalize(env) },
  message       = '%(env)s - 5XX alert'  % { env: grafana.utils.strings.capitalize(env) },
  notifications = notifications,
  noDataState   = 'no_data',
  period        = '0m',
  conditions    = [
    grafana.alertCondition.new(
      evaluatorParams = [ 15 ],
      evaluatorType   = 'gt',
      operatorType    = 'or',
      queryRefId      = 'ELB',
      queryTimeStart  = '15m',
      queryTimeEnd    = 'now',
      reducerType     = grafana.alert_reducers.Avg
    ),
  ],
);

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'HTTP 5xx Rate',
      datasource  = ds.cloudwatch,
    )
    .configure(defaults.configuration.timeseries)

    .setAlert(
      vars.environment,
      _alert(vars.namespace, vars.environment, vars.notifications)
    )

    .addTarget(targets.cloudwatch(
      alias       = 'ELB',
      datasource  = ds.cloudwatch,
      namespace   = 'AWS/ApplicationELB',
      metricName  = 'HTTPCode_ELB_5XX_Count',
      dimensions  = {
        LoadBalancer: vars.load_balancer
      },
      matchExact  = true,
      statistic   = 'Sum',
      refId       = 'ELB',
    ))
    .addTarget(targets.cloudwatch(
      alias       = 'ELB 500',
      datasource  = ds.cloudwatch,
      namespace   = 'AWS/ApplicationELB',
      metricName  = 'HTTPCode_ELB_500_Count',
      dimensions  = {
        LoadBalancer: vars.load_balancer
      },
      matchExact  = true,
      statistic   = 'Sum',
      refId       = 'ELB500',
    ))
    .addTarget(targets.cloudwatch(
      alias       = 'ELB 502',
      datasource  = ds.cloudwatch,
      namespace   = 'AWS/ApplicationELB',
      metricName  = 'HTTPCode_ELB_502_Count',
      dimensions  = {
        LoadBalancer: vars.load_balancer
      },
      matchExact  = true,
      statistic   = 'Sum',
      refId       = 'ELB502',
    ))
    .addTarget(targets.cloudwatch(
      alias       = 'ELB 503',
      datasource  = ds.cloudwatch,
      namespace   = 'AWS/ApplicationELB',
      metricName  = 'HTTPCode_ELB_503_Count',
      dimensions  = {
        LoadBalancer: vars.load_balancer
      },
      matchExact  = true,
      statistic   = 'Sum',
      refId       = 'ELB503',
    ))
    .addTarget(targets.cloudwatch(
      alias       = 'ELB 504',
      datasource  = ds.cloudwatch,
      namespace   = 'AWS/ApplicationELB',
      metricName  = 'HTTPCode_ELB_504_Count',
      dimensions  = {
        LoadBalancer: vars.load_balancer
      },
      matchExact  = true,
      statistic   = 'Sum',
      refId       = 'ELB504',
    ))

    .addTarget(targets.cloudwatch(
      alias       = 'Target',
      datasource  = ds.cloudwatch,
      namespace   = 'AWS/ApplicationELB',
      metricName  = 'HTTPCode_Target_5XX_Count',
      dimensions  = {
        LoadBalancer: vars.load_balancer
      },
      matchExact  = true,
      statistic   = 'Sum',
      refId       = 'Target',
    ))
}
