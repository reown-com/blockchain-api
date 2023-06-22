local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Cache-hit ratio',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries.withUnit('percent'))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_counter{}[$__rate_interval]))',
      refId       = "lookups",
      exemplar    = false,
      hide        = true,
    ))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_cache_hit_counter{}[$__rate_interval]))',
      refId       = "identity_lookup_cache_hit",
      exemplar    = false,
      hide        = true,
    ))
    .addTarget(targets.math(
      expr        = '($identity_lookup_cache_hit / $lookups) * 100',
      refId       = "Cache-hits",
    ))
}
