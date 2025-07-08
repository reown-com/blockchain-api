local grafana   = import '../../grafonnet-lib/grafana.libsonnet';
local defaults  = import '../../grafonnet-lib/defaults.libsonnet';

local panels         = grafana.panels;
local targets        = grafana.targets;
local alert          = grafana.alert;
local alertCondition = grafana.alertCondition;

local error_alert(vars) = alert.new(
  namespace   = 'Blockchain API',
  name        = "%s - Swaps availability" % vars.environment,
  message     = "%s - Swaps availability" % vars.environment,
  period      = '5m',
  frequency   = '1m',
  noDataState = 'alerting',
  notifications = vars.notifications,
  alertRuleTags = {
    'og_priority': 'P3',
  },
  
  conditions  = [
    alertCondition.new(
      evaluatorParams = [ 95 ],
      evaluatorType   = 'lt',
      operatorType    = 'or',
      queryRefId      = 'swaps_availability',
      queryTimeStart  = '5m',
      reducerType     = 'avg',
    ),
  ]
);

{
  new(ds, vars)::
    panels.timeseries(
      title       = 'Swaps availability',
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
    .setAlert(vars.environment, error_alert(vars))

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = '(1-(sum(rate(http_call_counter_total{code=~"5[0-9][0-9]", route="/v1/convert/tokens"}[$__rate_interval])) or vector(0))/(sum(rate(http_call_counter_total{route="/v1/convert/tokens"}[$__rate_interval]))))*100',
      refId         = 'swaps_tokens_availability',
      exemplar      = false,
      legendFormat  = 'Tokens list',
    ))

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = '(1-(sum(rate(http_call_counter_total{code=~"5[0-9][0-9]", route="/v1/convert/quotes"}[$__rate_interval])) or vector(0))/(sum(rate(http_call_counter_total{route="/v1/convert/quotes"}[$__rate_interval]))))*100',
      refId         = 'swaps_quotes_availability',
      exemplar      = false,
      legendFormat  = 'Quotes',
    ))

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = '(1-(sum(rate(http_call_counter_total{code=~"5[0-9][0-9]", route="/v1/convert/build-approve"}[$__rate_interval])) or vector(0))/(sum(rate(http_call_counter_total{route="/v1/convert/build-approve"}[$__rate_interval]))))*100',
      refId         = 'swaps_build_approve_availability',
      exemplar      = false,
      legendFormat  = 'Build approve',
    ))

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = '(1-(sum(rate(http_call_counter_total{code=~"5[0-9][0-9]", route="/v1/convert/build-transaction"}[$__rate_interval])) or vector(0))/(sum(rate(http_call_counter_total{route="/v1/convert/build-transaction"}[$__rate_interval]))))*100',
      refId         = 'swaps_build_transaction_availability',
      exemplar      = false,
      legendFormat  = 'Build transaction',
    ))

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = '(1-(sum(rate(http_call_counter_total{code=~"5[0-9][0-9]", route="/v1/convert/gas-price"}[$__rate_interval])) or vector(0))/(sum(rate(http_call_counter_total{route="/v1/convert/build-transaction"}[$__rate_interval]))))*100',
      refId         = 'swaps_gas_price_availability',
      exemplar      = false,
      legendFormat  = 'Gas price',
    ))

    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = '(1-(sum(rate(http_call_counter_total{code=~"5[0-9][0-9]", route="/v1/convert/allowance"}[$__rate_interval])) or vector(0))/(sum(rate(http_call_counter_total{route="/v1/convert/allowance"}[$__rate_interval]))))*100',
      refId         = 'swaps_allowance_availability',
      exemplar      = false,
      legendFormat  = 'Allowance check',
    ))

    // Hidden target for overall swaps availability alert
    .addTarget(targets.prometheus(
      datasource    = ds.prometheus,
      expr          = '(1-(sum(rate(http_call_counter_total{code=~"5[0-9][0-9]", route=~"/v1/convert/(tokens|quotes|build-approve|build-transaction|gas-price|allowance)"}[$__rate_interval])) or vector(0))/(sum(rate(http_call_counter_total{route=~"/v1/convert/(tokens|quotes|build-approve|build-transaction|gas-price|allowance)"}[$__rate_interval]))))*100',
      refId         = 'swaps_availability',
      exemplar      = false,
      legendFormat  = 'Overall availability',
      hide          = true,
    ))
}
