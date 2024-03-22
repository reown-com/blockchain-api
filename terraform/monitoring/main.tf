data "jsonnet_file" "dashboard" {
  source = "${path.module}/dashboard.jsonnet"

  ext_str = {
    dashboard_title = "BlockchainAPI - ${title(module.this.stage)}"
    dashboard_uid   = "blockchainapi-${module.this.stage}"

    prometheus_uid = grafana_data_source.prometheus.uid
    cloudwatch_uid = grafana_data_source.cloudwatch.uid

    environment   = module.this.stage
    notifications = jsonencode(var.notification_channels)

    ecs_service_name   = var.ecs_service_name
    ecs_task_family    = var.ecs_task_family
    load_balancer      = var.load_balancer_arn
    target_group       = var.ecs_target_group_arn
    redis_cluster_id   = var.redis_cluster_id
    log_group_app_name = var.log_group_app_name
    log_group_app_arn  = var.log_group_app_arn
    aws_account_id     = var.aws_account_id
  }
}

resource "grafana_dashboard" "main" {
  overwrite   = true
  message     = "Updated by Terraform"
  config_json = data.jsonnet_file.dashboard.rendered
}
