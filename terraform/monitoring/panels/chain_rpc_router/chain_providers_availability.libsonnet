local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars, chain)::
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
      datasource    = ds.prometheus,
      expr          = '(1-(sum(rate(no_providers_for_chain_counter_total{chain_id="%s"}[$__rate_interval])) or vector(0))/(sum(rate(found_provider_for_chain_counter_total{chain_id="%s"}[$__rate_interval])) + (sum(rate(no_providers_for_chain_counter_total{chain_id="%s"}[$__rate_interval])) or vector(0))))*100' % [chain.caip2, chain.caip2, chain.caip2],
      exemplar      = false,
      legendFormat  = 'Total',
      refId         = "TotalAvailability%s" % chain.caip2,
    ))
    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = '(sum by(provider) (rate(provider_finished_call_counter_total{chain_id="%s"}[$__rate_interval])) / sum by(provider) (rate(rpc_call_counter_total{chain_id="%s"}[$__rate_interval]))) * 100' % [chain.caip2, chain.caip2],
      exemplar      = false,
      legendFormat  = '{{provider}}',
      refId         = "ProviderAvailability%s" % chain.caip2,
    ))
} 