local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'CA response types rate',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries)
    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum(increase(ca_routes_found_total{}[$__rate_interval]))',
      exemplar      = false,
      legendFormat  = 'Routes found (success)',
    ))
    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum(increase(ca_insufficient_funds_total{}[$__rate_interval]))',
      exemplar      = false,
      legendFormat  = 'Insufficient funds',
    ))
    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum(increase(ca_no_bridging_needed_total{}[$__rate_interval]))',
      exemplar      = false,
      legendFormat  = 'No bridging needed',
    ))
    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum(increase(ca_no_routes_found_total{}[$__rate_interval]))',
      exemplar      = false,
      legendFormat  = 'No routes found',
    ))
}
