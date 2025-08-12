local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;
local alert           = grafana.alert;
local alertCondition  = grafana.alertCondition;

local _configuration = defaults.configuration.timeseries
  .withSoftLimit(
    axisSoftMin = 0,
    axisSoftMax = 5,
  );

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Healthy Hosts',
      datasource  = ds.cloudwatch,
    )
    .configure(_configuration)
      .setAlert(
        vars.environment,
        grafana.alert.new(
          namespace     = vars.namespace,
          name          = "%(env)s - Healthy hosts below min capacity" % { env: grafana.utils.strings.capitalize(vars.environment) },
          message       = '%(env)s - Healthy hosts below min capacity'  % { env: grafana.utils.strings.capitalize(vars.environment) },
          notifications = vars.notifications,
          noDataState   = 'no_data',
          period        = '0m',
          conditions    = [
            grafana.alertCondition.new(
              evaluatorParams = [ vars.app_autoscaling_min_capacity ],
              evaluatorType   = 'lt',
              operatorType    = 'or',
              queryRefId      = 'HealthyHosts',
              queryTimeStart  = '1m',
              queryTimeEnd    = 'now',
              reducerType     = grafana.alert_reducers.Avg,
            ),
          ],
        ),
      )
    

    .addTarget(targets.cloudwatch(
      datasource      = ds.cloudwatch,
      metricQueryType = grafana.target.cloudwatch.queryTypes.Query,

      dimensions    = {
        TargetGroup: vars.target_group
      },
      metricName    = 'HealthyHostCount',
      namespace     = 'AWS/ApplicationELB',
      refId         = 'HealthyHosts',
      sql           = {
        from: {
          property: {
            name: "AWS/ApplicationELB",
            type: "string"
          },
          type: "property"
        },
        select: {
          name: "MAX",
          parameters: [
            {
              name: "HealthyHostCount",
              type: "functionParameter"
            }
          ],
          type: "function"
        },
        where: {
          expressions: [
            {
              operator: {
                name: "=",
                value: vars.load_balancer
              },
              property: {
                name: "LoadBalancer",
                type: "string"
              },
              type: "operator"
            }
          ],
          type: "and"
        }
      },
      sqlExpression = "SELECT MAX(HealthyHostCount) FROM \"AWS/ApplicationELB\" WHERE LoadBalancer = '%s'" % [vars.load_balancer],
      statistic     = 'Maximum',
    ))
}
