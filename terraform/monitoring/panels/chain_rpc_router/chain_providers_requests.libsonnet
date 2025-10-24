local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars, chain)::
    panels.timeseries(
      title       = 'Requests',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries.withUnit('reqps'))

    .addTargets([
      targets.prometheus(
        datasource    = ds.prometheus,
        expr          = 'sum by(provider) (rate(provider_finished_call_counter_total{chain_id="%s"}[$__rate_interval]))' % chain.caip2,
        exemplar      = false,
        legendFormat  = '{{provider}} - Success',
        refId         = "ProviderRequestsSuccess%s" % chain.caip2,
      ),
      targets.prometheus(
        datasource    = ds.prometheus,
        expr          = 'sum by(provider) (rate(provider_failed_call_counter_total{chain_id="%s"}[$__rate_interval]))' % chain.caip2,
        exemplar      = false,
        legendFormat  = '{{provider}} - Failed',
        refId         = "ProviderRequestsFailed%s" % chain.caip2,
      ),
      targets.prometheus(
        datasource    = ds.prometheus,
        expr          = 'sum by(provider) (rate(rpc_call_counter_total{chain_id="%s"}[$__rate_interval]))' % chain.caip2,
        exemplar      = false,
        legendFormat  = '{{provider}} - Total',
        refId         = "ProviderRequestsTotal%s" % chain.caip2,
      ),
    ])
}