local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels         = grafana.panels;
local targets        = grafana.targets;
local alert          = grafana.alert;
local alertCondition = grafana.alertCondition;

local _configuration = defaults.configuration.timeseries
  .withUnit('s')
  .withSoftLimit(
    axisSoftMin = 0.4,
    axisSoftMax = 1.1,
  );

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Latency',
      datasource  = ds.prometheus,
    )
    .configure(_configuration)

    .setAlert(vars.environment, alert.new(
      namespace     = 'Blockchain API',
      name          = "%s - Identity cache high latency" % vars.environment,
      message       = "%s - Identity cache high latency" % vars.environment,
      period        = '5m',
      frequency     = '1m',
      noDataState   = 'no_data',
      notifications = vars.notifications,
      alertRuleTags = {
        'og_priority': 'P3',
      },
      conditions  = [
        alertCondition.new(
          evaluatorParams = [ 100 ],
          evaluatorType   = 'gt',
          operatorType    = 'or',
          queryRefId      = 'CacheLatency',
          queryTimeStart  = '5m',
          reducerType     = 'avg',
        ),
      ]
    ))

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum(rate(identity_lookup_latency_tracker_sum[$__rate_interval])) / sum(rate(identity_lookup_latency_tracker_count[$__rate_interval]))',
      refId         = 'EndpointLatency',
      legendFormat  = 'Endpoint',
    ))

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum(rate(identity_lookup_cache_latency_tracker_sum[$__rate_interval])) / sum(rate(identity_lookup_cache_latency_tracker_count[$__rate_interval]))',
      refId         = 'CacheLatency',
      legendFormat  = 'Cache',
    ))

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum(rate(identity_lookup_name_latency_tracker_sum[$__rate_interval])) / sum(rate(identity_lookup_name_latency_tracker_count[$__rate_interval]))',
      refId         = 'NameLatency',
      legendFormat  = 'Name',
    ))

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum(rate(identity_lookup_avatar_latency_tracker_sum[$__rate_interval])) / sum(rate(identity_lookup_avatar_latency_tracker_count[$__rate_interval]))',
      refId         = 'AvatarLatency',
      legendFormat  = 'Avatar',
    ))
}
