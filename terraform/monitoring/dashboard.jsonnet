local grafana     = import 'grafonnet-lib/grafana.libsonnet';
local panels      = import 'panels/panels.libsonnet';

local dashboard   = grafana.dashboard;
local row         = grafana.row;
local annotation  = grafana.annotation;
local layout      = grafana.layout;

local ds    = {
  prometheus: {
    type: 'prometheus',
    uid:  std.extVar('prometheus_uid'),
  },
  cloudwatch: {
    type: 'cloudwatch',
    uid:  std.extVar('cloudwatch_uid'),
  },
};
local vars  = {
  namespace:        'Blockchain API',
  environment:      std.extVar('environment'),
  notifications:    std.parseJson(std.extVar('notifications')),

  ecs_service_name:   std.extVar('ecs_service_name'),
  load_balancer:      std.extVar('load_balancer'),
  target_group:       std.extVar('target_group'),
  redis_cluster_id:   std.extVar('redis_cluster_id'),
  log_group_app_name: std.extVar('log_group_app_name'),
  log_group_app_arn:  std.extVar('log_group_app_arn'),
  aws_account_id:     std.extVar('aws_account_id'),
};

////////////////////////////////////////////////////////////////////////////////

local height    = 8;
local pos       = grafana.layout.pos(height);
local pos_short       = grafana.layout.pos(6);

////////////////////////////////////////////////////////////////////////////////

dashboard.new(
  title         = std.extVar('dashboard_title'),
  uid           = std.extVar('dashboard_uid'),
  editable      = true,
  graphTooltip  = dashboard.graphTooltips.sharedCrosshair,
)
.addAnnotation(
  annotation.new(
    target = {
      limit:    100,
      matchAny: false,
      tags:     [],
      type:     'dashboard',
    },
  )
)

.addPanels(layout.generate_grid([
  row.new('Application'),
    panels.ecs.availability(ds, vars)                { gridPos: pos._4 },
    panels.lb.error_5xx(ds, vars)                    { gridPos: pos._4 },
    panels.proxy.errors_non_provider(ds, vars)       { gridPos: pos._4 },
    panels.lb.error_5xx_logs(ds, vars)               { gridPos: pos._4 },
    panels.app.handlers_latency(ds, vars)            { gridPos: pos._2 },
    panels.app.handlers_rate(ds, vars)               { gridPos: pos._2 },

  row.new('ECS'),
    panels.ecs.memory(ds, vars)                      { gridPos: pos._3 },
    panels.ecs.cpu(ds, vars)                         { gridPos: pos._3 },

  row.new('RPC Proxy Chain Usage'),
    panels.usage.provider(ds, vars, 'Aurora')        { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'Arbitrum')      { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'Base')          { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'Binance')       { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'Infura')        { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'Near')          { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'Pokt')          { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'Publicnode')    { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'Quicknode')     { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'Zora')          { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'zkSync')        { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'Mantle')        { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'GetBlock')      { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'Berachain')     { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'Unichain')      { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'Lava')          { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'Morph')         { gridPos: pos._4 },

  row.new('RPC Proxy provider Weights'),
    panels.weights.provider(ds, vars, 'Aurora')      { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'Arbitrum')    { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'Base')        { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'Binance')     { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'Infura')      { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'Near')        { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'Pokt')        { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'Publicnode')  { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'Quicknode')   { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'Zora')        { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'zkSync')      { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'Mantle')      { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'GetBlock')    { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'Berachain')   { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'Unichain')    { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'Lava')        { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'Morph')       { gridPos: pos._4 },

  row.new('RPC Proxy providers Status Codes'),
    panels.status.provider(ds, vars, 'Aurora')       { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'Arbitrum')     { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'Base')         { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'Binance')      { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'Infura')       { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'Near')         { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'Pokt')         { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'Publicnode')   { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'Quicknode')    { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'Zora')         { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'zkSync')       { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'Mantle')       { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'GetBlock')     { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'Berachain')    { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'Unichain')     { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'Lava')         { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'Morph')        { gridPos: pos._4 },

  row.new('RPC Proxy Metrics'),
    panels.proxy.calls(ds, vars)                     { gridPos: pos._3 },
    panels.proxy.latency(ds, vars)                   { gridPos: pos._3 },
    panels.proxy.chains_unavailability(ds, vars)       { gridPos: pos._3 },
    panels.proxy.errors_provider(ds, vars)           { gridPos: pos._3 },
    panels.proxy.provider_retries(ds, vars)          { gridPos: pos._3 },
    panels.proxy.http_codes(ds, vars)                { gridPos: pos._3 },

  row.new('Non-RPC providers Status Codes'),
    panels.status.provider(ds, vars, 'Zerion')       { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'SolScan')      { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'OneInch')      { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'Coinbase')     { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'Bungee')       { gridPos: pos._4 },
  
  row.new('Non-RPC providers Latency'),
    panels.non_rpc.endpoints_latency(ds, vars, 'Zerion')       { gridPos: pos._4 },
    panels.non_rpc.endpoints_latency(ds, vars, 'SolScan')      { gridPos: pos._4 },
    panels.non_rpc.endpoints_latency(ds, vars, 'OneInch')      { gridPos: pos._4 },
    panels.non_rpc.endpoints_latency(ds, vars, 'Coinbase')     { gridPos: pos._4 },
    panels.non_rpc.endpoints_latency(ds, vars, 'Bungee')       { gridPos: pos._4 },

  row.new('Non-RPC providers Cache'),
    panels.non_rpc.cache_latency(ds, vars)      { gridPos: pos._2 },

  row.new('Projects registry'),
    panels.projects.rejected_projects(ds, vars)         { gridPos: pos._4 },
    panels.projects.quota_limited_projects(ds, vars)    { gridPos: pos._4 },
    panels.projects.cache_latency(ds, vars)             { gridPos: pos._4 },
    panels.projects.fetch_latency(ds, vars)             { gridPos: pos._4 },

  row.new('Rate limiting'),
    panels.rate_limiting.counter(ds, vars)      { gridPos: pos._3 },
    panels.rate_limiting.latency(ds, vars)      { gridPos: pos._3 },
    panels.rate_limiting.rate_limited(ds, vars) { gridPos: pos._3 },

  row.new('History Metrics'),
    panels.history.requests(ds, vars)               { gridPos: pos_short._3 },
    panels.history.latency(ds, vars)                { gridPos: pos_short._3 },
    panels.history.availability(ds, vars)           { gridPos: pos_short._3 },

  row.new('Identity resolver (ENS resolver) Metrics'),
    panels.identity.requests(ds, vars)               { gridPos: pos_short._2 },
    panels.identity.availability(ds, vars)           { gridPos: pos_short._2 },
    panels.identity.latency(ds, vars)                { gridPos: pos_short._2 },
    panels.identity.cache(ds, vars)                  { gridPos: pos_short._2 },
    panels.identity.usage(ds, vars)                  { gridPos: pos_short._2 },

  row.new('Account names (ENS gateway) Metrics'),
    panels.names.registered(ds, vars)                { gridPos: pos_short._3 },

  row.new('Redis'),
    panels.redis.cpu(ds, vars)                    { gridPos: pos._2 },
    panels.redis.memory(ds, vars)                 { gridPos: pos._2 },

  row.new('Load Balancer'),
    panels.lb.active_connections(ds, vars)        { gridPos: pos._2 },
    panels.lb.requests(ds, vars)                  { gridPos: pos._2 },

    panels.lb.healthy_hosts(ds, vars)             { gridPos: pos._3 },
    panels.lb.error_4xx(ds, vars)                 { gridPos: pos._3 },
    panels.lb.response_time(ds, vars)             { gridPos: pos._3 },

  row.new('IRN Client'),
    panels.irn.latency(ds, vars)        { gridPos: pos._2 },
]))
