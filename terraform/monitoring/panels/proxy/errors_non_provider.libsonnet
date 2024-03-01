local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels          = grafana.panels;
local targets         = grafana.targets;
local alert           = grafana.alert;
local alertCondition  = grafana.alertCondition;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Non-Provider Errors',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries)

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr        = 'round(sum(increase(http_call_counter_total{code=~"50[0-1]|50[5-9]|5[1-9][0-9]"}[5m])))',
      refId       = "non_provider_errors",
      exemplar    = true,
    )) 
}
