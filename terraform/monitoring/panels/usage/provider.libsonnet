local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars, provider)::
    panels.timeseries(
      title       = provider,
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries)

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr          = 'sum by(chain_id) (increase(provider_status_code_counter_total{provider="%s"}[5m]))' % provider,
      legendFormat  = '__auto',
    ))
}
