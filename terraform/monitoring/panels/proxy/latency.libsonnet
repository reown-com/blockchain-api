local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

local _configuration = defaults.configuration.timeseries
  .withUnit('ms')
  .withSoftLimit(
    axisSoftMin = 0.4,
    axisSoftMax = 1.1,
  );

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Latency',
      datasource  = ds.prometheus,
    )
    .configure(_configuration)

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr          = 'histogram_quantile(0.95, sum(rate(http_external_latency_tracker_bucket{}[$__rate_interval])) by (le, provider))',
      exemplar      = false,
      legendFormat  = '__auto',
    ))
}
