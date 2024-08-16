module "monitoring-role" {
  source          = "app.terraform.io/wallet-connect/monitoring-role/aws"
  version         = "1.0.2"
  context         = module.this
  remote_role_arn = var.monitoring_role_arn
}

resource "grafana_data_source" "prometheus" {
  type = "prometheus"
  name = "${module.this.stage}-${module.this.name}-amp"
  url  = var.prometheus_endpoint

  json_data_encoded = jsonencode({
    httpMethod         = "GET"
    sigV4Auth          = true
    sigV4AuthType      = "ec2_iam_role"
    sigV4Region        = module.this.region
    sigV4AssumeRoleArn = module.monitoring-role.iam_role_arn
  })

  depends_on = [module.monitoring-role]
}

resource "grafana_data_source" "cloudwatch" {
  type = "cloudwatch"
  name = "${module.this.stage}-${module.this.name}-cloudwatch"

  json_data_encoded = jsonencode({
    defaultRegion = module.this.region
    assumeRoleArn = module.monitoring-role.iam_role_arn
  })

  depends_on = [module.monitoring-role]
}
