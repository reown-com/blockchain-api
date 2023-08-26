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
  environment:      std.extVar('environment'),
  notifications:    std.parseJson(std.extVar('notifications')),

  ecs_service_name: std.extVar('ecs_service_name'),
  load_balancer:    std.extVar('load_balancer'),
  target_group:     std.extVar('target_group'),
  redis_cluster_id: std.extVar('redis_cluster_id'),
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
  row.new('ECS'),
    panels.ecs.availability(ds, vars)                { gridPos: pos._3 },
    panels.ecs.memory(ds, vars)                      { gridPos: pos._3 },
    panels.ecs.cpu(ds, vars)                         { gridPos: pos._3 },

  row.new('Chain Usage'),
    panels.usage.provider(ds, vars, 'Infura')        { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'zkSync')        { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'Publicnode')    { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'Omniatech')     { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'Binance')       { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'Pokt')          { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'Base')          { gridPos: pos._4 },
    panels.usage.provider(ds, vars, 'Zora')          { gridPos: pos._4 },

  row.new('Provider Weights'),
    panels.weights.provider(ds, vars, 'Infura')      { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'zkSync')      { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'Publicnode')  { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'Omniatech')   { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'Binance')     { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'Pokt')        { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'Base')        { gridPos: pos._4 },
    panels.weights.provider(ds, vars, 'Zora')        { gridPos: pos._4 },

  row.new('Status Codes'),
    panels.status.provider(ds, vars, 'Infura')       { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'zkSync')       { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'Publicnode')   { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'Omniatech')    { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'Binance')      { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'Pokt')         { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'Base')         { gridPos: pos._4 },
    panels.status.provider(ds, vars, 'Zora')         { gridPos: pos._4 },

  row.new('Proxy Metrics'),
    panels.proxy.calls(ds, vars)                     { gridPos: pos._2 },
    panels.proxy.latency(ds, vars)                   { gridPos: pos._2 },
    panels.proxy.errors_non_provider(ds, vars)       { gridPos: pos._3 },
    panels.proxy.errors_provider(ds, vars)           { gridPos: pos._3 },
    panels.proxy.rejected_projects(ds, vars)         { gridPos: pos._3 },
    panels.proxy.http_codes(ds, vars)                { gridPos: pos.two_thirds },
    panels.proxy.healthy_hosts(ds, vars)             { gridPos: pos._3 },

  row.new('Database'),
    panels.db.redis_cpu_memory(ds, vars)             { gridPos: pos._2 },

  row.new('Identity (ENS) Metrics'),
    panels.identity.requests(ds, vars)               { gridPos: pos_short._2 },
    panels.identity.availability(ds, vars)           { gridPos: pos_short._2 },
    panels.identity.latency(ds, vars)                { gridPos: pos_short._2 },
    panels.identity.cache(ds, vars)                  { gridPos: pos_short._2 },
    panels.identity.usage(ds, vars)                  { gridPos: pos_short._2 },
]))
