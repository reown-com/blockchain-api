module "monitoring" {
  source  = "./monitoring"
  context = module.this

  monitoring_role_arn   = data.terraform_remote_state.monitoring.outputs.grafana_workspaces.central.iam_role_arn
  notification_channels = var.notification_channels
  prometheus_endpoint   = aws_prometheus_workspace.prometheus.prometheus_endpoint
  ecs_service_name      = module.ecs.ecs_service_name
  ecs_task_family       = module.ecs.ecs_task_family
  ecs_target_group_arn  = module.ecs.target_group_arn
  load_balancer_arn     = module.ecs.load_balancer_arn_suffix
  redis_cluster_id      = module.redis.cluster_id
  log_group_app_name    = module.ecs.log_group_app_name
  log_group_app_arn     = module.ecs.log_group_app_arn
  aws_account_id        = data.aws_caller_identity.this.account_id
}
