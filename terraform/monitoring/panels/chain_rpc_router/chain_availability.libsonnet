local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels          = grafana.panels;
local targets         = grafana.targets;
// local alert           = grafana.alert;
// local alertCondition  = grafana.alertCondition;

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Availability',
      datasource  = ds.prometheus,
    )
    .configure(defaults.configuration.timeseries)
    // .setAlert(
    //   vars.environment,
    //   grafana.alert.new(
    //     namespace     = vars.namespace,
    //     name          = "%(env)s - RPC chain unavailability alert"  % { env: grafana.utils.strings.capitalize(vars.environment) },
    //     message       = '%(env)s - RPC chain unavailability alert'  % { env: grafana.utils.strings.capitalize(vars.environment) },
    //     notifications = vars.notifications,
    //     noDataState   = 'no_data',
    //     period        = '5m',
    //     conditions    = [
    //       grafana.alertCondition.new(
    //         evaluatorParams = [ 10 ],
    //         evaluatorType   = 'gt',
    //         operatorType    = 'or',
    //         queryRefId      = 'ChainsUnavailability',
    //         queryTimeStart  = '15m',
    //         queryTimeEnd    = 'now',
    //         reducerType     = grafana.alert_reducers.Avg
    //       ),
    //     ],
    //   ),
    // )

    .addTargets([
      targets.prometheus(
        datasource  = ds.prometheus,
        expr        = '(1-(sum(rate(no_providers_for_chain_counter_total{chain_id="%s"}[$__rate_interval])) or vector(0))/(sum(rate(found_provider_for_chain_counter_total{chain_id="%s"}[$__rate_interval])) + (sum(rate(no_providers_for_chain_counter_total{chain_id="%s"}[$__rate_interval])) or vector(0))))*100' % [chain.caip2, chain.caip2, chain.caip2],
        exemplar      = false,
        legendFormat  = chain.name,
        refId         = "ChainAvailability%s" % chain.caip2,
      )
      for chain in vars.chain_config.chains])
}
