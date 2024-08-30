local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels    = grafana.panels;
local targets   = grafana.targets;

local _configuration = defaults.configuration.timeseries
  .withUnit('ms');

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Fetch latency',
      datasource  = ds.prometheus,
    )
    .configure(_configuration)

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = 'sum(rate(project_data_registry_api_time_sum[$__rate_interval])) / sum(rate(project_data_registry_api_time_count[$__rate_interval]))',
      refId         = 'ProjectsRegistryFetchLatency',
      legendFormat  = 'Fetch',
    ))
}
