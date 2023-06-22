local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Availability',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries.withUnit('percent'))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_counter{}[$__rate_interval]))',
      refId       = "lookups",
      hide        = true,
    ))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_success_counter{}[$__rate_interval]))',
      refId       = "lookup_success",
      hide        = true,
    ))
    .addTarget(targets.math(
      expr        = '($lookup_success / $lookups) * 100',
      refId       = "Availability",
    ))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_cache_hit_counter{}[$__rate_interval]))',
      refId       = "lookup_cache_hit",
      hide        = true,
    ))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_name_success_counter{}[$__rate_interval]))',
      refId       = "lookup_name_success",
      hide        = true,
    ))
    .addTarget(targets.math(
      // Add lookup_cache_hit to not make our "name availability" appear to drop because it is divided by total lookups, not just name lookups
      // TODO Consider separate identity_lookup_name metric only created when performing a name lookup
      expr        = '(($lookup_cache_hit + $lookup_name_success) / $lookups) * 100',
      refId       = "Name availability",
    ))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_avatar_success_counter{}[$__rate_interval]))',
      refId       = "lookup_avatar_success",
      hide        = true,
    ))
    .addTarget(targets.math(
      // Add lookup_cache_hit to not make our "avatar availability" appear to drop because it is divided by total lookups, not just avatar lookups
      // TODO Consider separate identity_lookup_avatar metric only created when performing a avatar lookup
      expr        = '(($lookup_cache_hit + $lookup_avatar_success) / $lookups) * 100',
      refId       = "Avatar availability",
    ))
}
