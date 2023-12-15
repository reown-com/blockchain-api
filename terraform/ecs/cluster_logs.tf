resource "aws_cloudwatch_log_group" "cluster" {
  name              = "${module.this.id}-app-logs"
  kms_key_id        = var.cloudwatch_logs_key_arn
  retention_in_days = var.cloudwatch_retention_in_days
}

resource "aws_cloudwatch_log_group" "otel" {
  name              = "${module.this.id}-aws-otel-sidecar-collector"
  kms_key_id        = var.cloudwatch_logs_key_arn
  retention_in_days = var.cloudwatch_retention_in_days
}

resource "aws_cloudwatch_log_group" "prometheus_proxy" {
  name              = "${module.this.id}-sigv4-prometheus-proxy"
  kms_key_id        = var.cloudwatch_logs_key_arn
  retention_in_days = var.cloudwatch_retention_in_days
}
