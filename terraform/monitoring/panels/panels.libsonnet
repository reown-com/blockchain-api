{
  ecs: {
    availability:         (import 'ecs/availability.libsonnet'            ).new,
    cpu:                  (import 'ecs/cpu.libsonnet'                     ).new,
    memory:               (import 'ecs/memory.libsonnet'                  ).new,
  },

  weights: {
    provider:             (import 'weights/provider.libsonnet'            ).new,
  },

  usage: {
    provider:             (import 'usage/provider.libsonnet'              ).new,
  },

  status: {
    provider:             (import 'status/provider.libsonnet'             ).new,
  },

  proxy: {
    calls:                  (import 'proxy/calls.libsonnet'                 ).new,
    latency:                (import 'proxy/latency.libsonnet'               ).new,
    errors_non_provider:    (import 'proxy/errors_non_provider.libsonnet'   ).new,
    errors_provider:        (import 'proxy/errors_provider.libsonnet'       ).new,
    rejected_projects:      (import 'proxy/rejected_projects.libsonnet'     ).new,
    quota_limited_projects: (import 'proxy/quota_limited_projects.libsonnet').new,
    http_codes:             (import 'proxy/http_codes.libsonnet'            ).new,
    healthy_hosts:          (import 'proxy/healthy_hosts.libsonnet'         ).new,
  },

  db: {
    redis_cpu_memory:     (import 'db/redis_cpu_memory.libsonnet'         ).new,
  },

  identity: {
    requests:             (import 'identity/requests.libsonnet'           ).new,
    availability:         (import 'identity/availability.libsonnet'       ).new,
    latency:              (import 'identity/latency.libsonnet'            ).new,
    cache:                (import 'identity/cache.libsonnet'              ).new,
    usage:                (import 'identity/usage.libsonnet'              ).new,
  },

  history: {
    availability:         (import 'history/availability.libsonnet'        ).new,
    requests:             (import 'history/requests.libsonnet'            ).new,
    latency:              (import 'history/latency.libsonnet'             ).new,
  },

  lb: {
    active_connections:       (import 'lb/active_connections.libsonnet'         ).new,
    error_4xx:                (import 'lb/error_4xx.libsonnet'                  ).new,
    error_5xx:                (import 'lb/error_5xx.libsonnet'                  ).new,
    error_5xx_logs:           (import 'lb/error_5xx_logs.libsonnet'             ).new,
    healthy_hosts:            (import 'lb/healthy_hosts.libsonnet'              ).new,
    requests:                 (import 'lb/requests.libsonnet'                   ).new,
  },
}
