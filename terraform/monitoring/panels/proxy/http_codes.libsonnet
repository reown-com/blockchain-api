local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

local _configuration = defaults.configuration.timeseries
  .withUnit('cpm')
  .withSoftLimit(
    axisSoftMin = 0.4,
    axisSoftMax = 1.1,
  );

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'HTTP Response Codes',
      datasource  = ds.prometheus,
    )
    .configure(_configuration)

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr          = 'sum by (code)(rate(http_call_counter_total{}[5m]))',
      exemplar      = false,
      legendFormat  = '__auto',
    ))
}
