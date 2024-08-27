local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

local _configuration = defaults.configuration.timeseries
  .withUnit('cpm');

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Provider call retries',
      datasource  = ds.prometheus,
    )
    .configure(_configuration)

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr          = 'sum by (chain_id)(rate(rpc_call_retries_sum{}[$__rate_interval]))',
      exemplar      = false,
      legendFormat  = '__auto',
    ))
}
