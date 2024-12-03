local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'ChainID Availability',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries)

    .addTarget(targets.prometheus(
      datasource  = ds.prometheus,
      expr          = '((sum by(chain_id) (increase(rpc_call_counter_total{}[$__rate_interval])) – sum by(chain_id) (increase(no_providers_for_chain_counter{}[$__rate_interval]))) / sum by(chain_id) (increase(rpc_call_counter_total{}[$__rate_interval])) * 100)',
      exemplar      = false,
      legendFormat  = '__auto',
    ))
}
