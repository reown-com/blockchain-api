module "alerting" {
  source  = "./alerting"
  context = module.this

  webhook_cloudwatch_p2 = var.webhook_cloudwatch_p2
  webhook_prometheus_p2 = var.webhook_prometheus_p2

  ecs_cluster_name = module.ecs.ecs_cluster_name
  ecs_service_name = module.ecs.ecs_service_name

  redis_cluster_id = module.redis.cluster_id
}
