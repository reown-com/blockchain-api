local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels         = grafana.panels;
local targets        = grafana.targets;
local alert          = grafana.alert;
local alertCondition = grafana.alertCondition;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Target response time',
      datasource  = ds.cloudwatch,
    )
    .configure(defaults.configuration.timeseries.withUnit('s'))

    .setAlert(vars.environment, alert.new(
      namespace     = 'Blockchain API',
      name          = "%s - ELB High target response time" % vars.environment,
      message       = "%s - ELB High target response time" % vars.environment,
      period        = '5m',
      frequency     = '1m',
      noDataState   = 'no_data',
      notifications = vars.notifications,
      alertRuleTags = {
        'og_priority': 'P3',
      },
      conditions  = [
        alertCondition.new(
          evaluatorParams = [ 3 ],
          evaluatorType   = 'gt',
          operatorType    = 'or',
          queryRefId      = 'ELBTargetLatency',
          queryTimeStart  = '5m',
          reducerType     = 'avg',
        ),
      ]
    ))

    .addTarget(targets.cloudwatch(
      datasource      = ds.cloudwatch,
      namespace     = 'AWS/ApplicationELB',
      metricName    = 'TargetResponseTime',
      dimensions    = {
        LoadBalancer: vars.load_balancer
      },
      statistic     = 'Average',
      refId         = 'ELBTargetLatency',
    ))
}
