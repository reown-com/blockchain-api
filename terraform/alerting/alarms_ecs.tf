resource "aws_cloudwatch_metric_alarm" "ecs_cpu_utilization" {
  alarm_name        = "${local.alarm_prefix} - ECS CPU Utilization"
  alarm_description = "${local.alarm_prefix} - ECS CPU utilization is high (over ${var.ecs_cpu_threshold}%)"

  namespace = module.cloudwatch.namespaces.ECS
  dimensions = {
    ClusterName = var.ecs_cluster_name
    ServiceName = var.ecs_service_name
  }
  metric_name = module.cloudwatch.metrics.ECS.CPUUtilization

  evaluation_periods = local.evaluation_periods
  period             = local.period

  statistic           = module.cloudwatch.statistics.Average
  comparison_operator = module.cloudwatch.operators.GreaterThanOrEqualToThreshold
  threshold           = var.ecs_cpu_threshold
  treat_missing_data  = "breaching"

  alarm_actions             = [aws_sns_topic.cloudwatch_webhook.arn]
  insufficient_data_actions = [aws_sns_topic.cloudwatch_webhook.arn]
}

resource "aws_cloudwatch_metric_alarm" "ecs_mem_utilization" {
  alarm_name        = "${local.alarm_prefix} - ECS Memory Utilization"
  alarm_description = "${local.alarm_prefix} - ECS Memory utilization is high (over ${var.ecs_memory_threshold}%)"

  namespace = module.cloudwatch.namespaces.ECS
  dimensions = {
    ClusterName = var.ecs_cluster_name
    ServiceName = var.ecs_service_name
  }
  metric_name = module.cloudwatch.metrics.ECS.MemoryUtilization

  evaluation_periods = local.evaluation_periods
  period             = local.period

  statistic           = module.cloudwatch.statistics.Average
  comparison_operator = module.cloudwatch.operators.GreaterThanOrEqualToThreshold
  threshold           = var.ecs_memory_threshold
  treat_missing_data  = "breaching"

  alarm_actions             = [aws_sns_topic.cloudwatch_webhook.arn]
  insufficient_data_actions = [aws_sns_topic.cloudwatch_webhook.arn]
}
