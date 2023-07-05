local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Usage',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries.withUnit('percent'))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_success_counter_total{}[$__rate_interval]))',
      refId       = "lookups",
      exemplar    = false,
      hide        = true,
    ))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_name_present_counter_total{}[$__rate_interval]))',
      refId       = "name_present",
      exemplar    = false,
      hide        = true,
    ))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_avatar_present_counter_total{}[$__rate_interval]))',
      refId       = "avatar_present",
      exemplar    = false,
      hide        = true,
    ))

    .addTarget(targets.math(
      expr        = '($name_present / $lookups) * 100',
      refId       = "% of lookups with name",
    ))
    .addTarget(targets.math(
      expr        = '($avatar_present / $lookups) * 100',
      refId       = "% of lookups with avatar",
    ))
}
