local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars)::
    panels.timeseries(
      title       = "Requests distribution by provider (EVM)",
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries)
    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum (increase(provider_status_code_counter_total{provider="Dune", endpoint="evm_balances"}[$__rate_interval]))',
      legendFormat  = 'Dune',
    ))
    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum (increase(provider_status_code_counter_total{provider="Zerion", endpoint="positions"}[$__rate_interval]))',
      legendFormat  = 'Zerion',
    ))
}
