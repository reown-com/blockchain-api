local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Non-Provider Errors',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries)

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'round(sum(increase(http_call_counter{code=~\"5.+\"}[5m])))',
      refId       = "total",
      exemplar    = true,
      hide        = true,
    ))
    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'round(sum(increase(http_call_counter{code=\"502\"}[5m])))',
      refId       = "bad_gateway",
      exemplar    = true,
      hide        = true,
    ))
    .addTarget(targets.math(
      expr        = '$total - $bad_gateway',
      refId       = "Availability",
    ))
}
