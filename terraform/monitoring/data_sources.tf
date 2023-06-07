resource "grafana_data_source" "prometheus" {
  type = "prometheus"
  name = "${var.environment}-rpc-proxy-amp"
  url  = "https://aps-workspaces.eu-central-1.amazonaws.com/workspaces/${var.prometheus_workspace_id}/"

  json_data_encoded = jsonencode({
    httpMethod    = "GET"
    manageAlerts  = false
    sigV4Auth     = true
    sigV4AuthType = "ec2_iam_role"
    sigV4Region   = "eu-central-1"
  })
}

resource "grafana_data_source" "cloudwatch" {
  type = "cloudwatch"
  name = "${var.environment}-rpc-proxy-cloudwatch"

  json_data_encoded = jsonencode({
    defaultRegion = "eu-central-1"
  })
}
