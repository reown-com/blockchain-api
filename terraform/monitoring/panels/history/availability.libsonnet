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
      expr        = '(sum(rate(history_lookup_success_counter_total{}[$__rate_interval])) / sum(rate(history_lookup_counter_total{}[$__rate_interval]))) * 100',
      refId       = "availability",
    ))
}
