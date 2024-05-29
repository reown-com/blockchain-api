local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

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

    .addTarget(targets.cloudwatch(
      datasource      = ds.cloudwatch,
      metricQueryType = grafana.target.cloudwatch.queryTypes.Query,

      dimensions    = {
        TargetGroup: vars.target_group
      },
      metricName    = 'HealthyHostCount',
      namespace     = 'AWS/ApplicationELB',
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
