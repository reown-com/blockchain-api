local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels         = grafana.panels;
local targets        = grafana.targets;
local alert          = grafana.alert;
local alertCondition = grafana.alertCondition;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Availability',
      datasource  = ds.prometheus,
    )
    .configure(
      defaults.configuration.timeseries
        .withUnit('percent')
        .withSoftLimit(
          axisSoftMin = 98,
          axisSoftMax = 100,
        )
    )

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = '(1-(sum(rate(http_call_counter_total{aws_ecs_task_family="%s_rpc-proxy",code=~"5.+"}[5m])) or vector(0))/(sum(rate(http_call_counter_total{aws_ecs_task_family="%s_rpc-proxy"}[5m]))))*100' % [vars.environment, vars.environment],
      refId       = "availability",
      exemplar    = false,
      hide        = false,
    ))
}
