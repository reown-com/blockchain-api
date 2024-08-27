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
    .configure(
      defaults.configuration.timeseries
        .withUnit('percent')
        .withSoftLimit(
          axisSoftMin = 0,
          axisSoftMax = 100,
        )
    )

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_success_counter_total{}[$__rate_interval]))',
      refId       = "lookups",
      exemplar    = false,
      hide        = true,
    ))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_success_counter_total{source="cache"}[$__rate_interval]))',
      refId       = "cache_hits",
      exemplar    = false,
      hide        = true,
    ))
    .addTarget(targets.math(
      expr        = '($cache_hits / $lookups) * 100',
      refId       = "Cache-hits",
    ))
}
