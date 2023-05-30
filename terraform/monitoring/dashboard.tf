data "jsonnet_file" "dashboard" {
  source = "${path.module}/dashboard.jsonnet"

  ext_str = {
    dashboard_title = "${var.environment} - RPC Proxy"
    dashboard_uid   = "${var.environment}-rpc-proxy"

    prometheus_uid = grafana_data_source.prometheus.uid
    cloudwatch_uid = grafana_data_source.cloudwatch.uid

    environment      = var.environment
    notifications    = jsonencode(local.notifications)
    ecs_service_name = "${var.environment}_rpc-proxy-service"
    load_balancer    = local.load_balancer
    target_group     = local.target_group
    redis_cluster_id = var.redis_cluster_id
  }
}

resource "grafana_dashboard" "dashboard" {
  overwrite   = true
  message     = "Updated by Terraform"
  config_json = data.jsonnet_file.dashboard.rendered
}
