resource "aws_cloudwatch_metric_alarm" "redis_cpu_utilization" {
  alarm_name        = "${local.alarm_prefix} - Redis CPU Utilization"
  alarm_description = "${local.alarm_prefix} - Redis CPU utilization is high (over ${var.redis_cpu_threshold}%)"

  namespace = module.cloudwatch.namespaces.ElastiCache
  dimensions = {
    CacheClusterId = var.redis_cluster_id
  }
  metric_name = module.cloudwatch.metrics.ElastiCache.CPUUtilization

  evaluation_periods = local.evaluation_periods
  period             = local.period

  statistic           = module.cloudwatch.statistics.Average
  comparison_operator = module.cloudwatch.operators.GreaterThanOrEqualToThreshold
  threshold           = var.redis_cpu_threshold
  treat_missing_data  = "breaching"

  alarm_actions             = [aws_sns_topic.cloudwatch_webhook.arn]
  insufficient_data_actions = [aws_sns_topic.cloudwatch_webhook.arn]
}

resource "aws_cloudwatch_metric_alarm" "redis_available_memory" {
  alarm_name        = "${local.alarm_prefix} - Redis Available Memory"
  alarm_description = "${local.alarm_prefix} - Redis available memory is low (less than ${var.redis_memory_threshold}GiB)"

  namespace = module.cloudwatch.namespaces.ElastiCache
  dimensions = {
    CacheClusterId = var.redis_cluster_id
  }
  metric_name = module.cloudwatch.metrics.ElastiCache.FreeableMemory

  evaluation_periods = local.evaluation_periods
  period             = local.period

  statistic           = module.cloudwatch.statistics.Average
  comparison_operator = module.cloudwatch.operators.LessThanOrEqualToThreshold
  threshold           = var.redis_memory_threshold * pow(1000, 3)
  treat_missing_data  = "breaching"

  alarm_actions             = [aws_sns_topic.cloudwatch_webhook.arn]
  insufficient_data_actions = [aws_sns_topic.cloudwatch_webhook.arn]
}
