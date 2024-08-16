module "cloudwatch" {
  source  = "app.terraform.io/wallet-connect/cloudwatch-constants/aws"
  version = "1.0.0"
}

locals {
  alarm_prefix       = "${title(module.this.name)} - ${title(module.this.stage)}"
  evaluation_periods = 1
  period             = 60 * 5
}


#tfsec:ignore:aws-sns-enable-topic-encryption
resource "aws_sns_topic" "cloudwatch_webhook" {
  name         = "cloudwatch-webhook"
  display_name = "CloudWatch Webhook forwarding to BetterUptime"
}

resource "aws_sns_topic_subscription" "cloudwatch_webhook" {
  count = var.webhook_cloudwatch_p2 != "" ? 1 : 0

  endpoint  = var.webhook_cloudwatch_p2
  protocol  = "https"
  topic_arn = aws_sns_topic.cloudwatch_webhook.arn
}


#tfsec:ignore:aws-sns-enable-topic-encryption
resource "aws_sns_topic" "prometheus_webhook" {
  name         = "prometheus-webhook"
  display_name = "Prometheus Webhook forwarding to BetterUptime"
}

resource "aws_sns_topic_subscription" "prometheus_webhook" {
  count     = var.webhook_prometheus_p2 != "" ? 1 : 0
  endpoint  = var.webhook_prometheus_p2
  protocol  = "https"
  topic_arn = aws_sns_topic.prometheus_webhook.arn
}
