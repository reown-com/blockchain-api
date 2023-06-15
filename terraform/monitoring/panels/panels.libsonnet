{
  ecs: {
    availability:         (import 'ecs/availability.libsonnet'            ).new,
    cpu:                  (import 'ecs/cpu.libsonnet'                     ).new,
    memory:               (import 'ecs/memory.libsonnet'                  ).new,
  },

  usage: {
    provider:             (import 'usage/provider.libsonnet'              ).new,
  },

  status: {
    provider:             (import 'status/provider.libsonnet'             ).new,
  },

  proxy: {
    calls:                (import 'proxy/calls.libsonnet'                 ).new,
    latency:              (import 'proxy/latency.libsonnet'               ).new,
    errors_non_provider:  (import 'proxy/errors_non_provider.libsonnet'   ).new,
    errors_provider:      (import 'proxy/errors_provider.libsonnet'       ).new,
    rejected_projects:    (import 'proxy/rejected_projects.libsonnet'     ).new,
    http_codes:           (import 'proxy/http_codes.libsonnet'            ).new,
    healthy_hosts:        (import 'proxy/healthy_hosts.libsonnet'         ).new,
  },

  db: {
    redis_cpu_memory:     (import 'db/redis_cpu_memory.libsonnet'         ).new,
  },
}
