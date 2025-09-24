local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels         = grafana.panels;
local targets        = grafana.targets;

{
  new(ds, vars)::
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
        .withSpanNulls(true)
    )

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = '(1-(sum(rate(http_call_counter_total{code=~"5[0-9][0-9]", route="/v1/json-rpc"}[$__rate_interval])) or vector(0))/(sum(rate(http_call_counter_total{route="/v1/json-rpc"}[$__rate_interval]))))*100',
      exemplar      = false,
      legendFormat  = '__auto',
    ))
}
