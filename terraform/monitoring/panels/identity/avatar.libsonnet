local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Avatar usage',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries.withUnit('percent'))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_avatar_success_counter{}[$__rate_interval]))',
      refId       = "avatar_lookups",
      exemplar    = false,
      hide        = true,
    ))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_avatar_present_counter{}[$__rate_interval]))',
      refId       = "avatar_present",
      exemplar    = false,
      hide        = true,
    ))

    .addTarget(targets.math(
      expr        = '($avatar_present / $avatar_lookups) * 100',
      refId       = "Successful lookups with avatar",
    ))
}
