local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Gas estimations',
      datasource  = ds.prometheus,
    )
    .configure(
      defaults.configuration.timeseries
      .withSpanNulls(true)
    )
    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum by(chain_id) (rate(gas_estimation_sum[$__rate_interval])) / sum by(chain_id) (rate(gas_estimation_count[$__rate_interval]))',
      exemplar      = false,
      legendFormat  = '__auto',
    ))
}
