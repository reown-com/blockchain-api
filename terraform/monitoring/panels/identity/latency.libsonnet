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
      expr        = 'sum(rate(identity_lookup_latency_tracker_bucket{}[$__rate_interval]))',
      refId       = "Latency",
    ))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_cache_latency_tracker_bucket{}[$__rate_interval]))',
      refId       = "Cache latency",
    ))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_name_latency_tracker_bucket{}[$__rate_interval]))',
      refId       = "Name RPC latency",
    ))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_avatar_latency_tracker_bucket{}[$__rate_interval]))',
      refId       = "Avatar RPC latency",
    ))
}
