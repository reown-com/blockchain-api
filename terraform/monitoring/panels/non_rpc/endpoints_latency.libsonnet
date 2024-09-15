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
      datasource  = ds.prometheus,
      expr          = 'sum by(endpoint) (rate(http_external_latency_tracker_sum{provider="%s"}[$__rate_interval])) / sum by(endpoint) (rate(http_external_latency_tracker_count{provider="%s"}[$__rate_interval]))' % [provider, provider],
      legendFormat  = '__auto',
    ))
}
