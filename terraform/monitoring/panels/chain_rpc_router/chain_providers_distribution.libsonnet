local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

{
  new(ds, vars, chain)::
    panels.timeseries(
      title       = 'Distribution',
      datasource  = ds.prometheus,
    )
    // .configure({fieldConfig: {
    //     stacking: 'percent',
    //     fillOpacity: 100,
    // }, options: {}})
    .configure(defaults.configuration.timeseries.withUnit('percent'))
    .configure(defaults.configuration.timeseries.withSoftLimit(0, 100))

    .addTargets([
      targets.prometheus(
        datasource    = ds.prometheus,
        expr          = 'sum by(provider) (rate(provider_finished_call_counter_total{chain_id="%s"}[$__rate_interval]))' % [chain.caip2],
        exemplar      = false,
        legendFormat  = '{{provider}}',
        refId         = "ProviderDistribution%s" % chain.caip2,
      ),
    ])
    + {
        fieldConfig+: {
            defaults+: {
                custom+: {
                    stacking: {
                        mode: 'percent',
                    },
                    fillOpacity: 100,
                },
            }
        }
    }
} 