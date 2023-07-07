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
    .configure(
      defaults.configuration.timeseries
        .withUnit('percent')
        .withSoftLimit(
          axisSoftMin = 98,
          axisSoftMax = 100,
        )
    )

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_counter_total{}[$__rate_interval]))',
      refId       = "lookup",
      hide        = true,
    ))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_success_counter_total{}[$__rate_interval]))',
      refId       = "lookup_success",
      hide        = true,
    ))
    .addTarget(targets.math(
      expr        = '($lookup_success / $lookup) * 100',
      refId       = "Availability",
    ))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_name_counter_total{}[$__rate_interval]))',
      refId       = "lookup_name",
      hide        = true,
    ))
    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_name_success_counter_total{}[$__rate_interval]))',
      refId       = "lookup_name_success",
      hide        = true,
    ))
    .addTarget(targets.math(
      expr        = '($lookup_name_success / $lookup_name) * 100',
      refId       = "Name availability",
    ))

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_avatar_counter_total{}[$__rate_interval]))',
      refId       = "lookup_avatar",
      hide        = true,
    ))
    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'sum(rate(identity_lookup_avatar_success_counter_total{}[$__rate_interval]))',
      refId       = "lookup_avatar_success",
      hide        = true,
    ))
    .addTarget(targets.math(
      expr        = '($lookup_avatar_success / $lookup_avatar) * 100',
      refId       = "Avatar availability",
    ))
}
